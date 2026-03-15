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
    default: bool,
    hydrate_with: Option<syn::Path>,
    reconcile_with: Option<syn::Path>,
    rename: Option<String>,
    skip: bool,
    with: Option<syn::Path>,
}

fn parse_field_attrs(attrs: &[Attribute]) -> FieldAttrs {
    let mut result = FieldAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("loro") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                result.default = true;
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

            Ok(())
        });
    }

    result
}

/// Derive macro for `Hydrate` trait.
///
/// Generates an implementation that reads struct fields from a [`LoroValue::Map`],
/// or enum variants from a map with a `"tag"` discriminant key.
///
/// # Panics
///
/// Panics if the input is not a struct with named fields or an enum.
#[expect(clippy::too_many_lines)]
#[proc_macro_derive(Hydrate, attributes(loro))]
pub fn derive_hydrate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let crate_ident = crate_ident();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match &input.data {
        Data::Struct(data) => {
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

                    if attrs.default {
                        quote! {
                            #field_name: {
                                match map.get(#key) {
                                    Some(field_value) => #hydrate_expr,
                                    None => Default::default(),
                                }
                            },
                        }
                    } else {
                        quote! {
                            #field_name: {
                                let field_value = map.get(#key).ok_or_else(|| {
                                    #crate_ident::HydrateError::MissingField(#key.into())
                                })?;
                                #hydrate_expr
                            },
                        }
                    }
                })
                .collect();

            let expanded = quote! {
                impl #impl_generics #crate_ident::Hydrate for #name #ty_generics #where_clause {
                    fn hydrate(value: &#crate_ident::loro::LoroValue) -> Result<Self, #crate_ident::HydrateError> {
                        let #crate_ident::loro::LoroValue::Map(map) = value else {
                            return Err(#crate_ident::HydrateError::TypeMismatch {
                                expected: "Map".into(),
                                actual: format!("{:?}", value).into(),
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

        Data::Enum(data) => {
            let arms: Vec<_> = data
                .variants
                .iter()
                .map(|variant| {
                    let variant_ident = &variant.ident;
                    let attrs = parse_field_attrs(&variant.attrs);
                    let tag = attrs.rename.unwrap_or_else(|| variant_ident.to_string());

                    match &variant.fields {
                        Fields::Unit => {
                            quote! {
                                #tag => Ok(Self::#variant_ident),
                            }
                        }
                        Fields::Named(named) => {
                            let field_extractions: Vec<_> = named
                                .named
                                .iter()
                                .map(|f| {
                                    let field_name = f.ident.as_ref().expect("named field");
                                    let field_type = &f.ty;
                                    let f_attrs = parse_field_attrs(&f.attrs);
                                    let key = f_attrs
                                        .rename
                                        .unwrap_or_else(|| field_name.to_string());
                                    let hydrate_expr = if let Some(with) = f_attrs.with {
                                        quote! { #with::hydrate(field_value)? }
                                    } else if let Some(hw) = f_attrs.hydrate_with {
                                        quote! { #hw(field_value)? }
                                    } else {
                                        quote! { <#field_type as #crate_ident::Hydrate>::hydrate(field_value)? }
                                    };
                                    quote! {
                                        #field_name: {
                                            let field_value = inner.get(#key).ok_or_else(|| {
                                                #crate_ident::HydrateError::MissingField(#key.into())
                                            })?;
                                            #hydrate_expr
                                        },
                                    }
                                })
                                .collect();
                            quote! {
                                #tag => {
                                    let data = map.get(#tag).ok_or_else(|| {
                                        #crate_ident::HydrateError::MissingField(#tag.into())
                                    })?;
                                    let #crate_ident::loro::LoroValue::Map(inner) = data else {
                                        return Err(#crate_ident::HydrateError::TypeMismatch {
                                            expected: "Map".into(),
                                            actual: format!("{:?}", data).into(),
                                        });
                                    };
                                    Ok(Self::#variant_ident {
                                        #(#field_extractions)*
                                    })
                                }
                            }
                        }
                        Fields::Unnamed(unnamed) => {
                            let fields: Vec<_> = unnamed.unnamed.iter().collect();
                            if fields.len() != 1 {
                                return syn::Error::new_spanned(
                                    variant,
                                    "Hydrate enum only supports unit, named, or single-field tuple variants",
                                )
                                .to_compile_error();
                            }
                            let field_type = &fields[0].ty;
                            quote! {
                                #tag => {
                                    let data = map.get(#tag).ok_or_else(|| {
                                        #crate_ident::HydrateError::MissingField(#tag.into())
                                    })?;
                                    Ok(Self::#variant_ident(
                                        <#field_type as #crate_ident::Hydrate>::hydrate(&data)?
                                    ))
                                }
                            }
                        }
                    }
                })
                .collect();

            let expanded = quote! {
                impl #impl_generics #crate_ident::Hydrate for #name #ty_generics #where_clause {
                    fn hydrate(value: &#crate_ident::loro::LoroValue) -> Result<Self, #crate_ident::HydrateError> {
                        let #crate_ident::loro::LoroValue::Map(map) = value else {
                            return Err(#crate_ident::HydrateError::TypeMismatch {
                                expected: "Map".into(),
                                actual: format!("{:?}", value).into(),
                            });
                        };
                        let tag_value = map.get("tag").ok_or_else(|| {
                            #crate_ident::HydrateError::MissingField("tag".into())
                        })?;
                        let #crate_ident::loro::LoroValue::String(tag) = tag_value else {
                            return Err(#crate_ident::HydrateError::TypeMismatch {
                                expected: "String".into(),
                                actual: format!("{:?}", tag_value).into(),
                            });
                        };
                        match tag.as_str() {
                            #(#arms)*
                            other => Err(#crate_ident::HydrateError::Custom(
                                format!("unknown variant: {other}").into()
                            )),
                        }
                    }
                }
            };

            TokenStream::from(expanded)
        }

        Data::Union(_) => {
            syn::Error::new_spanned(&input, "Hydrate can only be derived for structs or enums")
                .to_compile_error()
                .into()
        }
    }
}

/// Derive macro for `Reconcile` trait.
///
/// Generates an implementation that writes struct fields to a [`LoroMap`],
/// or enum variants as a map with a `"tag"` discriminant key.
///
/// # Panics
///
/// Panics if the input is not a struct with named fields or an enum.
#[expect(clippy::too_many_lines)]
#[proc_macro_derive(Reconcile, attributes(loro))]
pub fn derive_reconcile(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let crate_ident = crate_ident();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match &input.data {
        Data::Struct(data) => {
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

        Data::Enum(data) => {
            let arms: Vec<_> = data
                .variants
                .iter()
                .map(|variant| {
                    let variant_ident = &variant.ident;
                    let attrs = parse_field_attrs(&variant.attrs);
                    let tag = attrs.rename.unwrap_or_else(|| variant_ident.to_string());

                    match &variant.fields {
                        Fields::Unit => {
                            quote! {
                                Self::#variant_ident => {
                                    map.insert("tag", #tag)?;
                                }
                            }
                        }
                        Fields::Named(named) => {
                            let field_names: Vec<_> = named
                                .named
                                .iter()
                                .map(|f| f.ident.as_ref().expect("named field"))
                                .collect();
                            let field_insertions: Vec<_> = named
                                .named
                                .iter()
                                .map(|f| {
                                    let field_name = f.ident.as_ref().expect("named field");
                                    let field_type = &f.ty;
                                    let f_attrs = parse_field_attrs(&f.attrs);
                                    let key = f_attrs
                                        .rename
                                        .unwrap_or_else(|| field_name.to_string());
                                    if let Some(with) = f_attrs.with {
                                        quote! { #with::reconcile(#field_name, &nested, #key)?; }
                                    } else if let Some(rw) = f_attrs.reconcile_with {
                                        quote! { #rw(#field_name, &nested, #key)?; }
                                    } else {
                                        quote! {
                                            <#field_type as #crate_ident::Reconcile>::reconcile_field(#field_name, &nested, #key)?;
                                        }
                                    }
                                })
                                .collect();
                            quote! {
                                Self::#variant_ident { #(#field_names),* } => {
                                    map.insert("tag", #tag)?;
                                    let nested = map.get_or_create_container(#tag, #crate_ident::loro::LoroMap::new())?;
                                    #(#field_insertions)*
                                }
                            }
                        }
                        Fields::Unnamed(unnamed) => {
                            let fields: Vec<_> = unnamed.unnamed.iter().collect();
                            if fields.len() != 1 {
                                return syn::Error::new_spanned(
                                    variant,
                                    "Reconcile enum only supports unit, named, or single-field tuple variants",
                                )
                                .to_compile_error();
                            }
                            let field_type = &fields[0].ty;
                            quote! {
                                Self::#variant_ident(v) => {
                                    map.insert("tag", #tag)?;
                                    <#field_type as #crate_ident::Reconcile>::reconcile_field(v, map, #tag)?;
                                }
                            }
                        }
                    }
                })
                .collect();

            let expanded = quote! {
                impl #impl_generics #crate_ident::Reconcile for #name #ty_generics #where_clause {
                    fn reconcile(&self, map: &#crate_ident::loro::LoroMap) -> Result<(), #crate_ident::ReconcileError> {
                        match self {
                            #(#arms)*
                        }
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

        Data::Union(_) => {
            syn::Error::new_spanned(&input, "Reconcile can only be derived for structs or enums")
                .to_compile_error()
                .into()
        }
    }
}
