//! Hydrate derive for structs.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    DataStruct,
    DeriveInput,
    Fields,
    Ident,
};

use crate::{
    attrs::{
        FieldAttrs,
        MissingStrategy,
        Strategy,
    },
    type_util::{
        is_option_type,
        is_vec,
    },
};

pub fn derive_hydrate_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let hydrate_body = match &data.fields {
        Fields::Named(fields) => derive_named_struct(fields)?,
        Fields::Unnamed(fields) => derive_tuple_struct(name, fields),
        Fields::Unit => derive_unit_struct(),
    };

    Ok(quote! {
        impl #impl_generics ::loro_surgeon::hydrate::Hydrate for #name #ty_generics #where_clause {
            #hydrate_body
        }
    })
}

fn derive_named_struct(fields: &syn::FieldsNamed) -> syn::Result<TokenStream> {
    let mut field_hydrations = Vec::new();
    let mut flatten_fields = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().expect("named field");
        let attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let loro_key = attrs.loro_key(&field_name.to_string());
        let field_ty = &field.ty;

        if attrs.strategy == Strategy::Flatten {
            flatten_fields.push((field_name.clone(), field_ty.clone()));
            continue;
        }

        field_hydrations.push(hydrate_one_field(field_name, field_ty, &loro_key, &attrs)?);
    }

    for (field_name, field_ty) in &flatten_fields {
        field_hydrations.push(quote! {
            #field_name: <#field_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_map(map)?,
        });
    }

    Ok(quote! {
        fn hydrate_map(map: &::loro::LoroMap) -> Result<Self, ::loro_surgeon::error::HydrateError> {
            Ok(Self { #(#field_hydrations)* })
        }
    })
}

fn hydrate_one_field(
    field_name: &Ident,
    field_ty: &syn::Type,
    loro_key: &str,
    attrs: &FieldAttrs,
) -> syn::Result<TokenStream> {
    if let Some(ref module) = attrs.with_module {
        let mod_path: syn::Path = syn::parse_str(module)?;
        return Ok(quote! { #field_name: #mod_path::hydrate(map, #loro_key)?, });
    }
    if let Some(ref func) = attrs.custom_hydrate {
        let func_path: syn::Path = syn::parse_str(func)?;
        return Ok(quote! { #field_name: #func_path(map, #loro_key)?, });
    }
    match attrs.strategy {
        Strategy::Json => hydrate_json_field(field_name, loro_key, attrs.missing.as_ref()),
        Strategy::Movable => Ok(quote! {
            #field_name: match map.get(#loro_key) {
                Some(::loro::ValueOrContainer::Container(::loro::Container::MovableList(list))) => {
                    <#field_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_movable_list(&list)?
                }
                Some(_) => return Err(::loro_surgeon::error::HydrateError::unexpected("movable_list", "other")),
                None => Default::default(),
            },
        }),
        Strategy::Plain if is_vec(field_ty) => Ok(quote! {
            #field_name: match map.get(#loro_key) {
                Some(::loro::ValueOrContainer::Container(::loro::Container::List(list))) => {
                    <#field_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_list(&list)?
                }
                Some(::loro::ValueOrContainer::Container(::loro::Container::MovableList(list))) => {
                    <#field_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_movable_list(&list)?
                }
                Some(other) => <#field_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate(&other)?,
                None => Default::default(),
            },
        }),
        Strategy::Plain => {
            hydrate_scalar_field(field_name, field_ty, loro_key, attrs.missing.as_ref())
        }
        Strategy::Flatten => unreachable!("flatten handled by caller"),
    }
}

fn hydrate_json_field(
    field_name: &Ident,
    loro_key: &str,
    missing: Option<&MissingStrategy>,
) -> syn::Result<TokenStream> {
    Ok(match missing {
        Some(MissingStrategy::Default) => quote! {
            #field_name: ::loro_surgeon::hydrate::hydrate_prop_json_or_default(map, #loro_key)?,
        },
        Some(MissingStrategy::Function(f)) => {
            let func_path: syn::Path = syn::parse_str(f)?;
            quote! {
                #field_name: ::loro_surgeon::hydrate::hydrate_prop_json_or_default(map, #loro_key)
                    .unwrap_or_else(|_| #func_path()),
            }
        }
        None => quote! {
            #field_name: ::loro_surgeon::hydrate::hydrate_prop_json(map, #loro_key)?,
        },
    })
}

fn hydrate_scalar_field(
    field_name: &Ident,
    field_ty: &syn::Type,
    loro_key: &str,
    missing: Option<&MissingStrategy>,
) -> syn::Result<TokenStream> {
    Ok(match missing {
        Some(MissingStrategy::Default) => quote! {
            #field_name: ::loro_surgeon::hydrate::hydrate_prop_or_default(map, #loro_key)?,
        },
        Some(MissingStrategy::Function(f)) => {
            let func_path: syn::Path = syn::parse_str(f)?;
            quote! {
                #field_name: ::loro_surgeon::hydrate::hydrate_prop_or_else(map, #loro_key, #func_path)?,
            }
        }
        None if is_option_type(field_ty) => quote! {
            #field_name: ::loro_surgeon::hydrate::hydrate_prop_or_default(map, #loro_key)?,
        },
        None => quote! {
            #field_name: ::loro_surgeon::hydrate::hydrate_prop(map, #loro_key)?,
        },
    })
}

fn derive_tuple_struct(name: &Ident, fields: &syn::FieldsUnnamed) -> TokenStream {
    if fields.unnamed.len() == 1 {
        let inner_ty = &fields.unnamed[0].ty;
        return quote! {
            fn hydrate(source: &::loro::ValueOrContainer) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate(source).map(#name)
            }
            fn hydrate_map(map: &::loro::LoroMap) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_map(map).map(#name)
            }
            fn hydrate_value(value: &::loro::LoroValue) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_value(value).map(#name)
            }
            fn hydrate_list(list: &::loro::LoroList) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_list(list).map(#name)
            }
            fn hydrate_movable_list(list: &::loro::LoroMovableList) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_movable_list(list).map(#name)
            }
            fn hydrate_null() -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_null().map(#name)
            }
            fn hydrate_bool(b: bool) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_bool(b).map(#name)
            }
            fn hydrate_i64(i: i64) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_i64(i).map(#name)
            }
            fn hydrate_f64(f: f64) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_f64(f).map(#name)
            }
            fn hydrate_string(s: &str) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_string(s).map(#name)
            }
            fn hydrate_binary(b: &[u8]) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_binary(b).map(#name)
            }
            fn hydrate_inline_list(items: &[::loro::LoroValue]) -> Result<Self, ::loro_surgeon::error::HydrateError> {
                <#inner_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate_inline_list(items).map(#name)
            }
        };
    }

    let field_count = fields.unnamed.len();
    let field_hydrations: Vec<_> = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let ty = &f.ty;
            quote! { ::loro_surgeon::hydrate::hydrate_list_item::<#ty>(list, #i)? }
        })
        .collect();

    quote! {
        fn hydrate_list(list: &::loro::LoroList) -> Result<Self, ::loro_surgeon::error::HydrateError> {
            if list.len() != #field_count {
                return Err(::loro_surgeon::error::HydrateError::unexpected(
                    concat!("list of length ", stringify!(#field_count)),
                    "list of different length",
                ));
            }
            Ok(Self(#(#field_hydrations),*))
        }
    }
}

fn derive_unit_struct() -> TokenStream {
    quote! {
        fn hydrate_null() -> Result<Self, ::loro_surgeon::error::HydrateError> {
            Ok(Self)
        }
    }
}
