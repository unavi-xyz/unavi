use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, parse_macro_input};

fn crate_ident() -> proc_macro2::TokenStream {
    match crate_name("loro-surgeon") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, proc_macro2::Span::call_site());
            quote!(#ident)
        }
        Err(_) => quote!(loro_surgeon),
    }
}

/// Field-level attributes parsed from `#[loro(...)]`.
#[derive(Default)]
struct FieldAttrs {
    with: Option<syn::Path>,
    hydrate_with: Option<syn::Path>,
    reconcile_with: Option<syn::Path>,
    rename: Option<String>,
    skip: bool,
}

fn parse_field_attrs(attrs: &[Attribute]) -> FieldAttrs {
    let mut result = FieldAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("loro") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                result.skip = true;
                return Ok(());
            }

            if meta.path.is_ident("with") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                result.with = Some(lit.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("hydrate_with") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                result.hydrate_with = Some(lit.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("reconcile_with") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                result.reconcile_with = Some(lit.parse()?);
                return Ok(());
            }

            if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                result.rename = Some(lit.value());
                return Ok(());
            }

            Ok(())
        });
    }

    result
}

/// Derive macro for `Hydrate` trait.
///
/// Generates an implementation that reads struct fields from a [`LoroValue::Map`].
///
/// # Panics
///
/// Panics if the input is not a struct with named fields.
#[proc_macro_derive(Hydrate, attributes(loro))]
pub fn derive_hydrate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let crate_ident = crate_ident();

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(&input, "Hydrate can only be derived for structs")
            .to_compile_error()
            .into();
    };

    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(&input, "Hydrate requires named fields")
            .to_compile_error()
            .into();
    };

    let field_extractions: Vec<_> = fields
        .named
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().expect("named field");
            let field_type = &f.ty;
            let attrs = parse_field_attrs(&f.attrs);

            if attrs.skip {
                return quote! {
                    #field_name: Default::default(),
                };
            }

            let key = attrs.rename.unwrap_or_else(|| field_name.to_string());

            let hydrate_expr = if let Some(with) = attrs.with {
                quote! { #with::hydrate(field_value)? }
            } else if let Some(hydrate_with) = attrs.hydrate_with {
                quote! { #hydrate_with(field_value)? }
            } else {
                quote! { <#field_type as #crate_ident::Hydrate>::hydrate(field_value)? }
            };

            quote! {
                #field_name: {
                    let field_value = map.get(#key).ok_or_else(|| {
                        #crate_ident::HydrateError::MissingField(#key.to_string())
                    })?;
                    #hydrate_expr
                },
            }
        })
        .collect();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #crate_ident::Hydrate for #name #ty_generics #where_clause {
            fn hydrate(value: &#crate_ident::loro::LoroValue) -> Result<Self, #crate_ident::HydrateError> {
                let #crate_ident::loro::LoroValue::Map(map) = value else {
                    return Err(#crate_ident::HydrateError::TypeMismatch {
                        path: stringify!(#name).to_string(),
                        expected: "Map".to_string(),
                        actual: format!("{:?}", value),
                    });
                };

                Ok(Self {
                    #(#field_extractions)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for `Reconcile` trait.
///
/// Generates an implementation that writes struct fields to a [`LoroMap`].
///
/// # Panics
///
/// Panics if the input is not a struct with named fields.
#[proc_macro_derive(Reconcile, attributes(loro))]
pub fn derive_reconcile(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let crate_ident = crate_ident();

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(&input, "Reconcile can only be derived for structs")
            .to_compile_error()
            .into();
    };

    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(&input, "Reconcile requires named fields")
            .to_compile_error()
            .into();
    };

    let field_insertions: Vec<_> = fields
        .named
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().expect("named field");
            let field_type = &f.ty;
            let attrs = parse_field_attrs(&f.attrs);

            if attrs.skip {
                return quote! {};
            }

            let key = attrs.rename.unwrap_or_else(|| field_name.to_string());

            if let Some(with) = attrs.with {
                quote! {
                    #with::reconcile(&self.#field_name, map, #key)?;
                }
            } else if let Some(reconcile_with) = attrs.reconcile_with {
                quote! {
                    #reconcile_with(&self.#field_name, map, #key)?;
                }
            } else {
                quote! {
                    <#field_type as #crate_ident::Reconcile>::reconcile_field(&self.#field_name, map, #key)?;
                }
            }
        })
        .collect();

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #crate_ident::Reconcile for #name #ty_generics #where_clause {
            fn reconcile(&self, map: &#crate_ident::loro::LoroMap) -> Result<(), #crate_ident::ReconcileError> {
                #(#field_insertions)*
                Ok(())
            }

            fn reconcile_field(&self, map: &#crate_ident::loro::LoroMap, key: &str) -> Result<(), #crate_ident::ReconcileError> {
                let nested = map.get_or_create_container(key, #crate_ident::loro::LoroMap::new())?;
                self.reconcile(&nested)
            }
        }
    };

    TokenStream::from(expanded)
}
