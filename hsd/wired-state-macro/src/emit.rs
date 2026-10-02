//! The `#[state]` expansion: which destination a field lives in, the dirty
//! set that tracks what changed, and the flush-then-adopt `sync`.

use anyhow::{
    Context,
    Result,
    bail,
};
use proc_macro2::{
    Ident,
    Literal,
    TokenStream,
};
use quote::{
    format_ident,
    quote,
};
use syn::{
    Attribute,
    Field,
    Fields,
    ItemStruct,
    Meta,
    punctuated::Punctuated,
};

/// Where a field's bytes live. Unannotated fields are plain wasm memory and
/// take part in nothing.
enum Destination {
    /// The session layer: replicated by state messages, attributed to whoever
    /// wrote each key, gone with the session.
    Session,
    /// The document: promoted by `commit` after the flush, durable only on a
    /// client holding the document's key.
    Document,
}

fn is_state_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "state")
}

fn parse_destination(field: &Ident, attr: &Attribute) -> Result<Destination> {
    let Meta::List(list) = &attr.meta else {
        bail!(
            "field `{field}`: `state` needs a destination, one of `session`, \
             `document`, or `device`"
        );
    };
    let dest: Ident = list
        .parse_args()
        .with_context(|| format!("field `{field}`: invalid `state` destination"))?;
    match dest.to_string().as_str() {
        "session" => Ok(Destination::Session),
        "document" => Ok(Destination::Document),
        "device" => bail!(
            "field `{field}`: `device` state has no host write path yet — \
             `wired:storage` is read-only, so declare the field `session` or \
             leave it unannotated"
        ),
        other => bail!(
            "field `{field}`: unknown `state` destination `{other}`, expected \
             `session`, `document`, or `device`"
        ),
    }
}

struct ParsedField {
    ident:       Ident,
    field:       Field,
    destination: Option<Destination>,
}

/// Expands a `#[state]` struct into itself, the dirty set that backs its
/// accessors, and the `sync` that flushes and adopts.
pub fn expand(item: ItemStruct) -> Result<TokenStream> {
    let (item, parsed) = parse_fields(item)?;
    let codegen = codegen(&item, &parsed);

    Ok(quote! {
        #item
        #codegen
    })
}

/// Strips the `#[state]` attributes off each field and rebuilds the struct
/// with its hidden dirty set in place.
fn parse_fields(item: ItemStruct) -> Result<(ItemStruct, Vec<ParsedField>)> {
    let ItemStruct {
        attrs: struct_attrs,
        vis,
        ident: name,
        generics,
        fields,
        ..
    } = item;

    let Fields::Named(named) = fields else {
        bail!("`state` needs a struct with named fields");
    };

    let mut parsed = Vec::new();
    for mut field in named.named {
        let ident = field.ident.clone().context("`state` needs named fields")?;
        let mut destination = None;
        let mut kept = Vec::new();
        for attr in std::mem::take(&mut field.attrs) {
            if is_state_attr(&attr) {
                destination = Some(parse_destination(&ident, &attr)?);
            } else {
                kept.push(attr);
            }
        }
        field.attrs = kept;
        parsed.push(ParsedField {
            ident,
            field,
            destination,
        });
    }

    let mut out_fields = Punctuated::new();
    for parsed_field in &parsed {
        out_fields.push(parsed_field.field.clone());
    }

    let dirty_ident = format_ident!("__{name}Dirty");
    if parsed.iter().any(|p| p.destination.is_some()) {
        let hidden: Field = syn::parse_quote! {
            __dirty: #dirty_ident
        };
        out_fields.push(hidden);
    }

    let item = ItemStruct {
        attrs: struct_attrs,
        vis,
        struct_token: syn::token::Struct::default(),
        ident: name,
        generics,
        fields: Fields::Named(syn::FieldsNamed {
            brace_token: named.brace_token,
            named:       out_fields,
        }),
        semi_token: None,
    };

    Ok((item, parsed))
}

/// The dirty set, one accessor pair per field, and the flush-and-adopt
/// `sync`.
fn codegen(item: &ItemStruct, parsed: &[ParsedField]) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let struct_ident = &item.ident;

    let stateful = parsed
        .iter()
        .filter(|p| p.destination.is_some())
        .collect::<Vec<_>>();
    let session_fields = parsed
        .iter()
        .filter(|p| matches!(p.destination, Some(Destination::Session)))
        .map(|p| &p.ident)
        .collect::<Vec<_>>();
    let document_fields = parsed
        .iter()
        .filter(|p| matches!(p.destination, Some(Destination::Document)))
        .map(|p| &p.ident)
        .collect::<Vec<_>>();

    let dirty_ident = format_ident!("__{struct_ident}Dirty");
    let dirty = dirty_struct(&dirty_ident, &stateful);
    let accessor_tokens = accessors(parsed);

    let sync = if stateful.is_empty() {
        quote!()
    } else {
        let flush = session_flush(&session_fields);
        let commit = document_flush(&document_fields);
        let adopt_tokens = adopt(&session_fields);
        let needs_document = !session_fields.is_empty() || !document_fields.is_empty();
        let document_binding = if needs_document {
            quote! {
                let document = crate::wired::scene::document::script_document()?;
            }
        } else {
            quote!()
        };
        quote! {
            /// Flushes the dirty set and adopts what present peers stated, one
            /// batch per destination and nothing per key. A flush that fails
            /// keeps its field dirty, so the next call retries it rather than
            /// reporting the value as written.
            ///
            /// `session` fields write the `shared` layer directly. `document`
            /// fields write `local` first, so the value exists to commit, then
            /// `commit` promotes it — a no-op unless this peer holds the
            /// document — and are not adopted back: there is no generic
            /// document read.
            pub fn sync(
                &mut self,
                prim: (u64, u64),
            ) -> ::anyhow::Result<()> {
                #document_binding
                #flush
                #commit
                #adopt_tokens
                self.__dirty = Default::default();
                Ok(())
            }
        }
    };

    let impl_block = quote! {
        impl #impl_generics #struct_ident #ty_generics #where_clause {
            #accessor_tokens
            #sync
        }
    };

    quote! {
        #dirty
        #impl_block
    }
}

/// One accessor pair per field; a stateful field's setter marks it dirty.
fn accessors(parsed: &[ParsedField]) -> TokenStream {
    let mut accessors = Vec::new();
    for parsed_field in parsed {
        let ident = &parsed_field.ident;
        let ty = &parsed_field.field.ty;
        let dirty = match &parsed_field.destination {
            Some(_) => quote! {
                self.__dirty.#ident = true;
            },
            None => quote!(),
        };
        let setter = format_ident!("set_{ident}", ident = ident);
        accessors.push(quote! {
            #[must_use]
            pub fn #ident(&self) -> &#ty {
                &self.#ident
            }

            pub fn #setter(&mut self, value: #ty) {
                self.#ident = value;
                #dirty
            }
        });
    }
    quote! { #(#accessors)* }
}

/// The per-field dirty flags backing the accessors.
fn dirty_struct(dirty_ident: &Ident, stateful: &[&ParsedField]) -> TokenStream {
    if stateful.is_empty() {
        return quote!();
    }
    let flags = stateful.iter().map(|p| &p.ident).collect::<Vec<_>>();
    quote! {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        struct #dirty_ident {
            #(
                #flags: bool,
            )*
        }
    }
}

/// The property a state field is stored at, in the `state` namespace.
fn field_key(ident: &Ident) -> Literal {
    Literal::string(&format!("state/{ident}"))
}

/// Encodes the dirty session fields as `custom` properties and applies them
/// to the `shared` layer in one batch.
fn session_flush(session_fields: &[&Ident]) -> TokenStream {
    if session_fields.is_empty() {
        return quote!();
    }
    let keys = session_fields
        .iter()
        .map(|ident| field_key(ident))
        .collect::<Vec<_>>();
    quote! {
        let mut shared_writes: Vec<crate::wired::scene::document::Edit> = Vec::new();
        #(
            if self.__dirty.#session_fields {
                shared_writes.push(crate::wired::scene::document::Edit::Set((
                    prim,
                    crate::wired::scene::properties::Property::Custom((
                        #keys.to_owned(),
                        ::postcard::to_allocvec(&self.#session_fields)?,
                    )),
                )));
            }
        )*

        if !shared_writes.is_empty() {
            document.apply(crate::wired::scene::document::Layer::Shared, &shared_writes)?;
        }
    }
}

/// Writes the dirty document fields to the `local` layer, then promotes
/// them with `commit`, so the value exists before `commit` is asked to keep
/// it.
fn document_flush(document_fields: &[&Ident]) -> TokenStream {
    if document_fields.is_empty() {
        return quote!();
    }
    let keys = document_fields
        .iter()
        .map(|ident| field_key(ident))
        .collect::<Vec<_>>();
    quote! {
        let mut local_writes: Vec<crate::wired::scene::document::Edit> = Vec::new();
        let mut commit_keys: Vec<((u64, u64), crate::wired::scene::properties::PropertyKey)> =
            Vec::new();
        #(
            if self.__dirty.#document_fields {
                local_writes.push(crate::wired::scene::document::Edit::Set((
                    prim,
                    crate::wired::scene::properties::Property::Custom((
                        #keys.to_owned(),
                        ::postcard::to_allocvec(&self.#document_fields)?,
                    )),
                )));
                commit_keys.push((
                    prim,
                    crate::wired::scene::properties::PropertyKey::Custom(#keys.to_owned()),
                ));
            }
        )*

        if !local_writes.is_empty() {
            document.apply(crate::wired::scene::document::Layer::Local, &local_writes)?;
            document.commit(&commit_keys)?;
        }
    }
}

/// Reads back every session field's composed value, key by key.
fn adopt(session_fields: &[&Ident]) -> TokenStream {
    if session_fields.is_empty() {
        return quote!();
    }
    let keys = session_fields
        .iter()
        .map(|ident| field_key(ident))
        .collect::<Vec<_>>();
    quote! {
        let values = document.get_many(&[
            #(
                (prim, crate::wired::scene::properties::PropertyKey::Custom(#keys.to_owned())),
            )*
        ]);
        let mut values = values.into_iter();
        #(
            if let Some(crate::wired::scene::properties::Property::Custom((_, bytes))) =
                values.next().flatten()
                && let Ok(value) = ::postcard::from_bytes(&bytes)
            {
                self.#session_fields = value;
            }
        )*
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_str;

    use super::*;

    fn render(source: &str) -> String {
        let item: ItemStruct = parse_str(source).expect("parse struct");
        let tokens = expand(item).expect("expand");
        let file = syn::parse2(tokens).expect("expansion parses as Rust");
        prettyplease::unparse(&file)
    }

    fn error_of(source: &str) -> String {
        let item: ItemStruct = parse_str(source).expect("parse struct");
        expand(item).expect_err("expand fails").to_string()
    }

    #[test]
    fn session_field_flushes_and_adopts() {
        let rendered = render(
            r"
            #[wired_guest::state]
            #[derive(Default)]
            struct Counter {
                #[state(session)]
                ticks: u64,
            }
            ",
        );

        assert!(rendered.contains("pub fn sync"));
        assert!(rendered.contains("Layer::Shared, &shared_writes"));
        assert!(rendered.contains(r#""state/ticks".to_owned()"#));
        assert!(rendered.contains("::postcard::to_allocvec(&self.ticks)?"));
        assert!(rendered.contains(".get_many("));
        assert!(rendered.contains("::postcard::from_bytes(&bytes)"));
        assert!(rendered.contains("self.__dirty.ticks = true"));
        assert!(rendered.contains("struct __CounterDirty"));
        assert!(
            rendered.contains("let document = crate::wired::scene::document::script_document()?;")
        );
        assert!(!rendered.contains("document.commit("));
    }

    #[test]
    fn document_field_commits_after_the_flush() {
        let rendered = render(
            r"
            #[wired_guest::state]
            struct Bookmark {
                #[state(document)]
                page: u32,
            }
            ",
        );

        assert!(!rendered.contains("Layer::Shared"));
        assert!(
            rendered.contains("let document = crate::wired::scene::document::script_document()?;")
        );
        assert!(rendered.contains("Layer::Local, &local_writes"));
        assert!(rendered.contains("document.commit(&commit_keys)?"));
        assert!(rendered.contains(r#""state/page".to_owned()"#));
        assert!(!rendered.contains("get_many("));
        assert!(!rendered.contains("from_bytes"));
    }

    #[test]
    fn both_destinations_flush_session_then_document() {
        let rendered = render(
            r"
            #[wired_guest::state]
            struct Mixed {
                #[state(session)]
                live: u8,
                #[state(document)]
                kept: u8,
            }
            ",
        );

        let session = rendered.find("Layer::Shared").expect("session flush");
        let commit = rendered.find("document.commit").expect("document flush");
        assert!(
            session < commit,
            "session flushes before the commit promotes it"
        );
        assert!(rendered.contains("struct __MixedDirty"));
        assert!(rendered.contains("self.__dirty.live = true"));
        assert!(rendered.contains("self.__dirty.kept = true"));
    }

    #[test]
    fn an_unannotated_field_is_left_out_of_sync() {
        let rendered = render(
            r"
            #[wired_guest::state]
            struct Quiet {
                notes: String,
            }
            ",
        );

        assert!(rendered.contains("pub fn notes(&self) -> &String"));
        assert!(rendered.contains("pub fn set_notes(&mut self, value: String)"));
        assert!(!rendered.contains("__dirty"));
        assert!(!rendered.contains("pub fn sync"));
    }

    #[test]
    fn accessors_are_generated_for_every_field() {
        let rendered = render(
            r"
            #[wired_guest::state]
            #[derive(Default)]
            struct Gate {
                #[state(session)]
                link: Option<u32>,
                plain: u64,
            }
            ",
        );

        assert!(rendered.contains("pub fn link(&self) -> &Option<u32>"));
        assert!(rendered.contains("pub fn set_link(&mut self, value: Option<u32>)"));
        assert!(rendered.contains("pub fn plain(&self) -> &u64"));
        assert!(rendered.contains("pub fn set_plain(&mut self, value: u64)"));
    }

    #[test]
    fn the_hidden_dirty_field_is_private() {
        let rendered = render(
            r"
            #[wired_guest::state]
            struct Keep {
                #[state(session)]
                value: u8,
            }
            ",
        );
        assert!(rendered.contains("__dirty: __KeepDirty"));
        assert!(!rendered.contains("pub __dirty"));
    }

    #[test]
    fn device_is_rejected() {
        let error = error_of(
            r"
            #[wired_guest::state]
            struct Local {
                #[state(device)]
                volume: u8,
            }
            ",
        );
        assert!(error.contains("`device` state has no host write path yet"));
    }

    #[test]
    fn an_unknown_destination_is_rejected() {
        let error = error_of(
            r"
            #[wired_guest::state]
            struct Unknown {
                #[state(elsewhere)]
                value: u8,
            }
            ",
        );
        assert!(error.contains("unknown `state` destination `elsewhere`"));
    }

    #[test]
    fn a_state_attribute_without_a_destination_is_rejected() {
        let error = error_of(
            r"
            #[wired_guest::state]
            struct Bare {
                #[state]
                value: u8,
            }
            ",
        );
        assert!(error.contains("`state` needs a destination"));
    }

    #[test]
    fn a_struct_without_named_fields_is_rejected() {
        let item: ItemStruct = parse_str("struct Point(u8, u8);").expect("parse struct");
        let error = expand(item).expect_err("expand fails").to_string();
        assert!(error.contains("named fields"));
    }
}
