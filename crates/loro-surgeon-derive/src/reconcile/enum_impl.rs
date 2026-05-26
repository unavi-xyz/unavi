//! Reconcile derive for enums.

use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote,
};
use syn::{
    DataEnum,
    DeriveInput,
    Fields,
    Ident,
    Variant,
};

use crate::attrs::{
    FieldAttrs,
    Strategy,
};

struct EnumKey {
    decl:           TokenStream,
    ty:             TokenStream,
    key_fn:         TokenStream,
    hydrate_key_fn: TokenStream,
}

pub fn derive_reconcile_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let all_unit = data
        .variants
        .iter()
        .all(|v| matches!(v.fields, Fields::Unit));
    let has_keys = data.variants.iter().any(variant_has_key);

    let match_arms: Vec<_> = data
        .variants
        .iter()
        .map(|v| reconcile_variant_arm(name, v, all_unit))
        .collect();

    let key = if has_keys {
        generate_enum_key(name, data)
    } else {
        EnumKey {
            decl:           TokenStream::new(),
            ty:             quote! { ::loro_surgeon::reconcile::NoKey },
            key_fn:         TokenStream::new(),
            hydrate_key_fn: TokenStream::new(),
        }
    };
    let EnumKey {
        decl,
        ty: key_type,
        key_fn,
        hydrate_key_fn,
    } = key;

    quote! {
        #decl

        impl #impl_generics ::loro_surgeon::reconcile::Reconcile for #name #ty_generics #where_clause {
            type Key = #key_type;

            fn reconcile<R: ::loro_surgeon::reconcile::Reconciler>(&self, r: R) -> Result<(), ::loro_surgeon::error::ReconcileError> {
                match self {
                    #(#match_arms)*
                }
            }

            #key_fn
            #hydrate_key_fn
        }
    }
}

fn variant_has_key(v: &Variant) -> bool {
    matches!(&v.fields, Fields::Named(fields) if fields
        .named
        .iter()
        .any(|f| FieldAttrs::from_attrs(&f.attrs).is_ok_and(|a| a.is_key)))
}

fn reconcile_variant_arm(name: &Ident, v: &Variant, all_unit: bool) -> TokenStream {
    let variant_name = &v.ident;
    let variant_str = variant_name.to_string();

    match &v.fields {
        Fields::Unit if all_unit => quote! {
            #name::#variant_name => ::loro_surgeon::reconcile::Reconciler::str(r, #variant_str),
        },
        Fields::Unit => quote! {
            #name::#variant_name => {
                let mut m = ::loro_surgeon::reconcile::Reconciler::map(r)?;
                m.retain(|k| k == #variant_str)?;
                m.entry(#variant_str, &String::from(#variant_str))?;
                Ok(())
            }
        },
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => quote! {
            #name::#variant_name(inner) => {
                let mut m = ::loro_surgeon::reconcile::Reconciler::map(r)?;
                m.retain(|k| k == #variant_str)?;
                m.entry(#variant_str, inner)?;
                Ok(())
            }
        },
        Fields::Unnamed(fields) => {
            reconcile_tuple_variant(name, variant_name, &variant_str, fields)
        }
        Fields::Named(fields) => reconcile_named_variant(name, variant_name, &variant_str, fields),
    }
}

fn reconcile_tuple_variant(
    name: &Ident,
    variant_name: &Ident,
    variant_str: &str,
    fields: &syn::FieldsUnnamed,
) -> TokenStream {
    let bindings: Vec<_> = (0..fields.unnamed.len())
        .map(|i| Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()))
        .collect();
    let pattern = quote! { #(#bindings),* };
    let list_entries: Vec<_> = bindings
        .iter()
        .enumerate()
        .map(|(i, binding)| quote! { l.insert(#i, #binding)?; })
        .collect();
    quote! {
        #name::#variant_name(#pattern) => {
            let mut m = ::loro_surgeon::reconcile::Reconciler::map(r)?;
            m.retain(|k| k == #variant_str)?;
            let prop_r = ::loro_surgeon::reconcile::PropReconciler::map_put(m.map.clone(), #variant_str.to_string());
            let mut l = ::loro_surgeon::reconcile::Reconciler::list(prop_r)?;
            while l.len() > 0 {
                l.delete(0)?;
            }
            #(#list_entries)*
            Ok(())
        }
    }
}

fn reconcile_named_variant(
    name: &Ident,
    variant_name: &Ident,
    variant_str: &str,
    fields: &syn::FieldsNamed,
) -> TokenStream {
    let field_names: Vec<_> = fields
        .named
        .iter()
        .map(|f| f.ident.as_ref().expect("named field"))
        .collect();

    let field_entries: Vec<_> = fields
        .named
        .iter()
        .map(reconcile_named_variant_field)
        .collect();

    let pattern = quote! { #(#field_names),* };

    quote! {
        #name::#variant_name { #pattern } => {
            let mut m = ::loro_surgeon::reconcile::Reconciler::map(r)?;
            m.retain(|k| k == #variant_str)?;
            let prop_r = ::loro_surgeon::reconcile::PropReconciler::map_put(m.map.clone(), #variant_str.to_string());
            let mut inner_map = ::loro_surgeon::reconcile::Reconciler::map(prop_r)?;
            #(#field_entries)*
            Ok(())
        }
    }
}

fn reconcile_named_variant_field(f: &syn::Field) -> TokenStream {
    let field_name = f.ident.as_ref().expect("named field");
    let attrs = FieldAttrs::from_attrs(&f.attrs).unwrap_or_default();
    let loro_key = attrs.loro_key(&field_name.to_string());
    if attrs.strategy == Strategy::Json {
        quote! {
            {
                let json_str: String = serde_json::to_string(#field_name)
                    .map_err(::loro_surgeon::error::ReconcileError::from)?;
                inner_map.entry(#loro_key, &json_str)?;
            }
        }
    } else {
        quote! { inner_map.entry(#loro_key, #field_name)?; }
    }
}

fn generate_enum_key(name: &Ident, data: &DataEnum) -> EnumKey {
    let key_name = format_ident!("__{}Key", name);
    let mut variants = Vec::new();
    let mut extract_arms = Vec::new();
    let mut hydrate_arms = Vec::new();

    for variant in &data.variants {
        let parts = key_parts_for_variant(name, &key_name, variant);
        variants.push(parts.variant);
        extract_arms.push(parts.extract);
        hydrate_arms.push(parts.hydrate);
    }

    let decl = quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum #key_name {
            #(#variants),*
        }
    };
    let key_fn = quote! {
        fn key(&self) -> ::loro_surgeon::reconcile::LoadKey<Self::Key> {
            match self {
                #(#extract_arms)*
            }
        }
    };
    let string_match = string_key_match(data, &key_name);
    let hydrate_key_fn = quote! {
        fn hydrate_key(source: &::loro::ValueOrContainer) -> Result<::loro_surgeon::reconcile::LoadKey<Self::Key>, ::loro_surgeon::error::ReconcileError> {
            match source {
                ::loro::ValueOrContainer::Container(::loro::Container::Map(map)) => {
                    #(#hydrate_arms)*
                    Ok(::loro_surgeon::reconcile::LoadKey::KeyNotFound)
                }
                #string_match
                _ => Ok(::loro_surgeon::reconcile::LoadKey::KeyNotFound),
            }
        }
    };

    EnumKey {
        decl,
        ty: quote! { #key_name },
        key_fn,
        hydrate_key_fn,
    }
}

struct KeyVariantParts {
    variant: TokenStream,
    extract: TokenStream,
    hydrate: TokenStream,
}

fn key_parts_for_variant(name: &Ident, key_name: &Ident, variant: &Variant) -> KeyVariantParts {
    let variant_name = &variant.ident;
    let variant_str = variant_name.to_string();

    match &variant.fields {
        Fields::Named(fields) => {
            let key_field = fields
                .named
                .iter()
                .find(|f| FieldAttrs::from_attrs(&f.attrs).is_ok_and(|a| a.is_key));
            key_field.map_or_else(
                || unkeyed_named_variant_parts(name, key_name, variant_name, &variant_str, fields),
                |kf| {
                    keyed_named_variant_parts(
                        name,
                        key_name,
                        variant_name,
                        &variant_str,
                        fields,
                        kf,
                    )
                },
            )
        }
        Fields::Unnamed(_) => KeyVariantParts {
            variant: quote! { #variant_name },
            extract: quote! {
                #name::#variant_name(..) => {
                    ::loro_surgeon::reconcile::LoadKey::Found(#key_name::#variant_name)
                }
            },
            hydrate: quote! {
                if map.get(#variant_str).is_some() {
                    return Ok(::loro_surgeon::reconcile::LoadKey::Found(#key_name::#variant_name));
                }
            },
        },
        Fields::Unit => KeyVariantParts {
            variant: quote! { #variant_name },
            extract: quote! {
                #name::#variant_name => {
                    ::loro_surgeon::reconcile::LoadKey::Found(#key_name::#variant_name)
                }
            },
            hydrate: quote! {
                if map.get(#variant_str).is_some() {
                    return Ok(::loro_surgeon::reconcile::LoadKey::Found(#key_name::#variant_name));
                }
            },
        },
    }
}

fn keyed_named_variant_parts(
    name: &Ident,
    key_name: &Ident,
    variant_name: &Ident,
    variant_str: &str,
    fields: &syn::FieldsNamed,
    kf: &syn::Field,
) -> KeyVariantParts {
    let key_field_name = kf.ident.as_ref().expect("named field");
    let key_field_ty = &kf.ty;
    let attrs = FieldAttrs::from_attrs(&kf.attrs).unwrap_or_default();
    let loro_key = attrs.loro_key(&key_field_name.to_string());

    let other_fields: Vec<_> = fields
        .named
        .iter()
        .filter(|f| f.ident.as_ref().expect("named field") != key_field_name)
        .map(|f| {
            let n = f.ident.as_ref().expect("named field");
            quote! { #n: _ }
        })
        .collect();

    KeyVariantParts {
        variant: quote! { #variant_name(#key_field_ty) },
        extract: quote! {
            #name::#variant_name { #key_field_name, #(#other_fields),* } => {
                ::loro_surgeon::reconcile::LoadKey::Found(#key_name::#variant_name(#key_field_name.clone()))
            }
        },
        hydrate: quote! {
            if let Some(::loro::ValueOrContainer::Container(::loro::Container::Map(inner))) = map.get(#variant_str)
                && let Some(voc) = inner.get(#loro_key)
                && let Ok(k) = <#key_field_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate(&voc)
            {
                return Ok(::loro_surgeon::reconcile::LoadKey::Found(#key_name::#variant_name(k)));
            }
        },
    }
}

fn unkeyed_named_variant_parts(
    name: &Ident,
    key_name: &Ident,
    variant_name: &Ident,
    variant_str: &str,
    fields: &syn::FieldsNamed,
) -> KeyVariantParts {
    let other_fields: Vec<_> = fields
        .named
        .iter()
        .map(|f| {
            let n = f.ident.as_ref().expect("named field");
            quote! { #n: _ }
        })
        .collect();

    KeyVariantParts {
        variant: quote! { #variant_name },
        extract: quote! {
            #name::#variant_name { #(#other_fields),* } => {
                ::loro_surgeon::reconcile::LoadKey::Found(#key_name::#variant_name)
            }
        },
        hydrate: quote! {
            if map.get(#variant_str).is_some() {
                return Ok(::loro_surgeon::reconcile::LoadKey::Found(#key_name::#variant_name));
            }
        },
    }
}

fn string_key_match(data: &DataEnum, key_name: &Ident) -> TokenStream {
    let arms: Vec<_> = data
        .variants
        .iter()
        .filter(|v| matches!(v.fields, Fields::Unit))
        .map(|v| {
            let variant_name = &v.ident;
            let variant_str = variant_name.to_string();
            quote! {
                #variant_str => return Ok(::loro_surgeon::reconcile::LoadKey::Found(#key_name::#variant_name)),
            }
        })
        .collect();
    if arms.is_empty() {
        TokenStream::new()
    } else {
        quote! {
            ::loro::ValueOrContainer::Value(::loro::LoroValue::String(s)) => {
                match s.as_ref() {
                    #(#arms)*
                    _ => {}
                }
                Ok(::loro_surgeon::reconcile::LoadKey::KeyNotFound)
            }
        }
    }
}
