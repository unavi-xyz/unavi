//! Hydrate derive for enums.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Fields, Ident, Variant};

use crate::{attrs::{FieldAttrs, MissingStrategy, Strategy}, type_util::{is_option_type, is_vec}};

pub fn derive_hydrate_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let has_unit_variants = data.variants.iter().any(|v| matches!(v.fields, Fields::Unit));
    let has_data_variants = data.variants.iter().any(|v| !matches!(v.fields, Fields::Unit));

    let hydrate_string_fn = hydrate_string_fn(name, data, has_unit_variants);
    let map_variant_arms: Vec<_> = data
        .variants
        .iter()
        .map(|v| hydrate_variant_arm(name, v))
        .collect();
    let hydrate_map_fn = hydrate_map_fn(name, &map_variant_arms, has_unit_variants, has_data_variants);
    let hydrate_value_override = hydrate_value_override(has_unit_variants, has_data_variants);

    quote! {
        impl #impl_generics ::loro_surgeon::hydrate::Hydrate for #name #ty_generics #where_clause {
            #hydrate_value_override
            #hydrate_string_fn
            #hydrate_map_fn
        }
    }
}

fn hydrate_string_fn(name: &Ident, data: &DataEnum, has_unit_variants: bool) -> TokenStream {
    if !has_unit_variants {
        return TokenStream::new();
    }
    let unit_arms: Vec<_> = data
        .variants
        .iter()
        .filter(|v| matches!(v.fields, Fields::Unit))
        .map(|v| {
            let variant_name = &v.ident;
            let variant_str = variant_name.to_string();
            quote! { #variant_str => Ok(#name::#variant_name), }
        })
        .collect();
    let name_str = name.to_string();
    quote! {
        fn hydrate_string(s: &str) -> Result<Self, ::loro_surgeon::error::HydrateError> {
            match s {
                #(#unit_arms)*
                _ => Err(::loro_surgeon::error::HydrateError::unexpected(
                    concat!(#name_str, " variant"),
                    "unknown variant",
                )),
            }
        }
    }
}

fn hydrate_map_fn(
    name: &Ident,
    arms: &[TokenStream],
    has_unit: bool,
    has_data: bool,
) -> TokenStream {
    if !(has_unit || has_data) {
        return TokenStream::new();
    }
    let name_str = name.to_string();
    quote! {
        fn hydrate_map(map: &::loro::LoroMap) -> Result<Self, ::loro_surgeon::error::HydrateError> {
            #(#arms)*
            Err(::loro_surgeon::error::HydrateError::unexpected(
                concat!(#name_str, " variant"),
                "unknown variant in map",
            ))
        }
    }
}

fn hydrate_value_override(has_unit: bool, has_data: bool) -> TokenStream {
    if has_data || !has_unit {
        return TokenStream::new();
    }
    quote! {
        fn hydrate(source: &::loro::ValueOrContainer) -> Result<Self, ::loro_surgeon::error::HydrateError> {
            match source {
                ::loro::ValueOrContainer::Value(v) => Self::hydrate_value(v),
                ::loro::ValueOrContainer::Container(::loro::Container::Map(m)) => Self::hydrate_map(m),
                _ => Err(::loro_surgeon::error::HydrateError::unexpected("string or map", "other container")),
            }
        }
    }
}

fn hydrate_variant_arm(name: &Ident, v: &Variant) -> TokenStream {
    let variant_name = &v.ident;
    let variant_str = variant_name.to_string();

    match &v.fields {
        Fields::Unit => quote! {
            if let Some(::loro::ValueOrContainer::Value(::loro::LoroValue::String(s))) = map.get(#variant_str)
                && s.as_ref() == #variant_str
            {
                return Ok(#name::#variant_name);
            }
        },
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let inner_ty = &fields.unnamed[0].ty;
            quote! {
                if let Some(inner) = map.get(#variant_str) {
                    return <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate(&inner)
                        .map(#name::#variant_name);
                }
            }
        }
        Fields::Unnamed(fields) => {
            let hydrations: Vec<_> = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let ty = &f.ty;
                    quote! { ::loro_surgeon::hydrate::hydrate_list_item::<#ty>(&list, #i)? }
                })
                .collect();
            quote! {
                if let Some(::loro::ValueOrContainer::Container(::loro::Container::List(list))) = map.get(#variant_str) {
                    return Ok(#name::#variant_name(#(#hydrations),*));
                }
            }
        }
        Fields::Named(fields) => {
            let hydrations: Vec<_> = fields
                .named
                .iter()
                .map(hydrate_named_variant_field)
                .collect();
            quote! {
                if let Some(::loro::ValueOrContainer::Container(::loro::Container::Map(inner_map))) = map.get(#variant_str) {
                    return Ok(#name::#variant_name { #(#hydrations)* });
                }
            }
        }
    }
}

fn hydrate_named_variant_field(f: &syn::Field) -> TokenStream {
    let field_name = f.ident.as_ref().expect("named field");
    let attrs = FieldAttrs::from_attrs(&f.attrs).unwrap_or_default();
    let loro_key = attrs.loro_key(&field_name.to_string());
    let ty = &f.ty;

    if is_vec(ty) {
        return quote! {
            #field_name: match inner_map.get(#loro_key) {
                Some(::loro::ValueOrContainer::Container(::loro::Container::List(list))) => {
                    <#ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_list(&list)?
                }
                Some(other) => <#ty as ::loro_surgeon::hydrate::Hydrate>::hydrate(&other)?,
                None => Default::default(),
            },
        };
    }
    if attrs.strategy == Strategy::Json {
        return match &attrs.missing {
            Some(MissingStrategy::Default) => quote! {
                #field_name: ::loro_surgeon::hydrate::hydrate_prop_json_or_default(&inner_map, #loro_key)?,
            },
            Some(MissingStrategy::Function(fn_str)) => {
                let func_path: syn::Path = syn::parse_str(fn_str).expect("valid path");
                quote! {
                    #field_name: ::loro_surgeon::hydrate::hydrate_prop_json_or_default(&inner_map, #loro_key)
                        .unwrap_or_else(|_| #func_path()),
                }
            }
            None => quote! {
                #field_name: ::loro_surgeon::hydrate::hydrate_prop_json(&inner_map, #loro_key)?,
            },
        };
    }
    if is_option_type(ty) {
        return quote! {
            #field_name: ::loro_surgeon::hydrate::hydrate_prop_or_default(&inner_map, #loro_key)?,
        };
    }
    quote! {
        #field_name: ::loro_surgeon::hydrate::hydrate_prop(&inner_map, #loro_key)?,
    }
}
