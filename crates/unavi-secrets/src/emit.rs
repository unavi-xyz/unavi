use std::collections::BTreeMap;

use anyhow::{
    Context,
    Result,
    bail,
};
use proc_macro2::{
    Literal,
    TokenStream,
};
use quote::quote;
use secretspec::codegen::IrField;
use syn::Ident;

/// Renders the struct and constructor for `fields`.
///
/// A secret the manifest gives a value becomes a [`String`] carrying it, which
/// the build environment may override so a packaged build can name its own
/// deployment. A secret with no declared value is never compiled in, only ever
/// read from the environment the binary runs under.
pub fn accessor(
    profile: &str,
    fields: &[IrField],
    defaults: &BTreeMap<String, String>,
) -> Result<TokenStream> {
    let mut declarations = Vec::new();
    let mut initializers = Vec::new();

    for field in fields {
        if field.as_path {
            bail!(
                "secret '{}' is `as_path`, which only a runtime resolve can materialize",
                field.name
            );
        }

        declarations.push(declaration(field, defaults)?);
        initializers.push(initializer(field, defaults)?);
    }

    let summary =
        format!(" Configuration read from `secretspec.toml` under the `{profile}` profile.");

    Ok(quote! {
        #[doc = #summary]
        pub struct Secrets {
            #(#declarations)*
        }

        impl Secrets {
            /// Reads each secret from the environment, falling back on the value
            /// its profile declared.
            #[must_use]
            pub fn load() -> Self {
                Self {
                    #(#initializers)*
                }
            }
        }
    })
}

fn declaration(field: &IrField, defaults: &BTreeMap<String, String>) -> Result<TokenStream> {
    let ident = ident(&field.name)?;

    let doc = field.description.as_ref().map(|text| {
        let text = format!(" {text}");
        quote! { #[doc = #text] }
    });

    let ty = if defaults.contains_key(&field.name) {
        quote!(String)
    } else {
        quote!(Option<String>)
    };

    Ok(quote! {
        #doc
        pub #ident: #ty,
    })
}

fn initializer(field: &IrField, defaults: &BTreeMap<String, String>) -> Result<TokenStream> {
    let ident = ident(&field.name)?;
    let name = Literal::string(&field.name);

    let read = defaults.get(&field.name).map_or_else(
        || quote! { ::std::env::var(#name).ok() },
        |declared| {
            let declared = Literal::string(declared);
            quote! {
                ::std::env::var(#name)
                    .ok()
                    .or_else(|| ::core::option_env!(#name).map(|value| value.to_owned()))
                    .unwrap_or_else(|| #declared.to_owned())
            }
        },
    );

    Ok(quote! { #ident: #read, })
}

fn ident(name: &str) -> Result<Ident> {
    syn::parse_str(&name.to_lowercase())
        .with_context(|| format!("secret '{name}' does not name a Rust field"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str) -> IrField {
        IrField {
            name:        name.to_owned(),
            optional:    false,
            as_path:     false,
            description: None,
        }
    }

    fn render(fields: &[IrField], defaults: &BTreeMap<String, String>) -> String {
        let tokens = accessor("default", fields, defaults).expect("render accessor");
        let file = syn::parse2(tokens).expect("expansion parses as Rust");
        prettyplease::unparse(&file)
    }

    #[test]
    fn a_declared_value_is_compiled_in() {
        let defaults = BTreeMap::from([("HOST".to_owned(), "remote".to_owned())]);
        let rendered = render(&[field("HOST")], &defaults);

        assert!(rendered.contains("pub host: String,"));
        assert!(rendered.contains(r#"option_env!("HOST")"#));
        assert!(rendered.contains(r#"unwrap_or_else(|| "remote".to_owned())"#));
    }

    /// A secret the manifest never gave a value is one the build has no
    /// business carrying, so no build-time lookup stands behind it.
    #[test]
    fn an_undeclared_value_is_read_only_at_runtime() {
        let rendered = render(&[field("TOKEN")], &BTreeMap::new());

        assert!(rendered.contains("pub token: Option<String>,"));
        assert!(rendered.contains(r#"env::var("TOKEN").ok()"#));
        assert!(!rendered.contains("option_env!"));
    }

    #[test]
    fn a_description_spanning_lines_still_parses() {
        let mut described = field("HOST");
        described.description = Some("first\nsecond \"quoted\"".to_owned());

        render(&[described], &BTreeMap::new());
    }

    #[test]
    fn a_value_needing_escapes_stays_a_valid_literal() {
        let defaults = BTreeMap::from([("HOST".to_owned(), "a\"b\\c\nd".to_owned())]);
        let rendered = render(&[field("HOST")], &defaults);

        assert!(rendered.contains(r#""a\"b\\c\nd""#));
    }

    #[test]
    fn a_secret_naming_a_keyword_is_rejected() {
        assert!(accessor("default", &[field("TYPE")], &BTreeMap::new()).is_err());
    }

    #[test]
    fn an_as_path_secret_is_rejected() {
        let mut path = field("CERT");
        path.as_path = true;

        assert!(accessor("default", &[path], &BTreeMap::new()).is_err());
    }
}
