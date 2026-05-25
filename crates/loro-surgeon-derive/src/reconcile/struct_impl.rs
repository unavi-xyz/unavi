//! Reconcile derive for structs.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    DataStruct,
    DeriveInput,
    Fields,
    Ident,
};

use crate::attrs::{
    ContainerAttrs,
    FieldAttrs,
    Strategy,
};

/// Pieces emitted into the `impl Reconcile`: body, key type, `key` fn,
/// `hydrate_key` fn.
struct ReconcileImpl {
    body:           TokenStream,
    key_type:       TokenStream,
    key_fn:         TokenStream,
    hydrate_key_fn: TokenStream,
}

pub fn derive_reconcile_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let container_attrs = ContainerAttrs::from_attrs(&input.attrs)?;

    let imp = match &data.fields {
        Fields::Named(fields) => derive_named_struct(fields)?,
        Fields::Unnamed(fields) => derive_tuple_struct(fields),
        Fields::Unit => derive_unit_struct(),
    };
    let ReconcileImpl {
        body,
        key_type,
        key_fn,
        hydrate_key_fn,
    } = imp;

    let doc_sync_impl = container_attrs.root.as_ref().map_or_else(TokenStream::new, |root_key| {
        quote! {
            impl #impl_generics ::loro_surgeon::doc_sync::DocSync for #name #ty_generics #where_clause {
                const ROOT_KEY: &'static str = #root_key;
            }
        }
    });

    Ok(quote! {
        impl #impl_generics ::loro_surgeon::reconcile::Reconcile for #name #ty_generics #where_clause {
            type Key = #key_type;
            #body
            #key_fn
            #hydrate_key_fn
        }
        #doc_sync_impl
    })
}

fn derive_named_struct(fields: &syn::FieldsNamed) -> syn::Result<ReconcileImpl> {
    let mut field_reconciliations = Vec::new();
    let mut key_field: Option<(Ident, syn::Type)> = None;

    for field in &fields.named {
        let field_name = field.ident.as_ref().expect("named field");
        let attrs = FieldAttrs::from_attrs(&field.attrs)?;
        let loro_key = attrs.loro_key(&field_name.to_string());

        if attrs.is_key {
            key_field = Some((field_name.clone(), field.ty.clone()));
        }

        field_reconciliations.push(reconcile_one_field(field_name, &loro_key, &attrs)?);
    }

    let body = quote! {
        fn reconcile<R: ::loro_surgeon::reconcile::Reconciler>(&self, r: R) -> Result<(), ::loro_surgeon::error::ReconcileError> {
            let mut m = ::loro_surgeon::reconcile::Reconciler::map(r)?;
            #(#field_reconciliations)*
            Ok(())
        }
    };

    let (key_type, key_fn, hydrate_key_fn) = key_field.map_or_else(
        || (quote! { ::loro_surgeon::reconcile::NoKey }, TokenStream::new(), TokenStream::new()),
        |(key_name, key_ty)| {
            let key_str = key_name.to_string();
            (
                quote! { #key_ty },
                quote! {
                    fn key(&self) -> ::loro_surgeon::reconcile::LoadKey<Self::Key> {
                        ::loro_surgeon::reconcile::LoadKey::Found(self.#key_name.clone())
                    }
                },
                quote! {
                    fn hydrate_key(source: &::loro::ValueOrContainer) -> Result<::loro_surgeon::reconcile::LoadKey<Self::Key>, ::loro_surgeon::error::ReconcileError> {
                        let ::loro::ValueOrContainer::Container(::loro::Container::Map(map)) = source else {
                            return Ok(::loro_surgeon::reconcile::LoadKey::KeyNotFound);
                        };
                        let Some(voc) = map.get(#key_str) else {
                            return Ok(::loro_surgeon::reconcile::LoadKey::KeyNotFound);
                        };
                        let value = <#key_ty as ::loro_surgeon::hydrate::Hydrate>::hydrate(&voc)
                            .map_err(|_| ::loro_surgeon::error::ReconcileError::TypeMismatch {
                                expected: "key value",
                                found: "incompatible type",
                            })?;
                        Ok(::loro_surgeon::reconcile::LoadKey::Found(value))
                    }
                },
            )
        },
    );

    Ok(ReconcileImpl {
        body,
        key_type,
        key_fn,
        hydrate_key_fn,
    })
}

fn reconcile_one_field(
    field_name: &Ident,
    loro_key: &str,
    attrs: &FieldAttrs,
) -> syn::Result<TokenStream> {
    if attrs.strategy == Strategy::Flatten {
        return Ok(quote! {
            {
                let inner_reconciler = ::loro_surgeon::reconcile::RootReconciler::new(m.map.clone());
                ::loro_surgeon::reconcile::Reconcile::reconcile(&self.#field_name, inner_reconciler)?;
            }
        });
    }
    if let Some(ref module) = attrs.with_module {
        let mod_path: syn::Path = syn::parse_str(module)?;
        return Ok(quote! { #mod_path::reconcile(&self.#field_name, &mut m, #loro_key)?; });
    }
    if let Some(ref func) = attrs.custom_reconcile {
        let func_path: syn::Path = syn::parse_str(func)?;
        return Ok(quote! { #func_path(&self.#field_name, &mut m, #loro_key)?; });
    }
    Ok(match attrs.strategy {
        Strategy::Json => quote! {
            {
                let json_str: String = serde_json::to_string(&self.#field_name)
                    .map_err(::loro_surgeon::error::ReconcileError::from)?;
                m.entry(#loro_key, &json_str)?;
            }
        },
        Strategy::Movable => quote! {
            {
                let reconciler = ::loro_surgeon::reconcile::PropReconciler::map_put(
                    m.map.clone(), #loro_key.to_string(),
                );
                ::loro_surgeon::reconcile::list::reconcile_vec_movable(&self.#field_name, reconciler)?;
            }
        },
        Strategy::Plain => quote! { m.entry(#loro_key, &self.#field_name)?; },
        Strategy::Flatten => unreachable!("flatten handled above"),
    })
}

fn derive_tuple_struct(fields: &syn::FieldsUnnamed) -> ReconcileImpl {
    if fields.unnamed.len() == 1 {
        let inner_ty = &fields.unnamed[0].ty;
        return ReconcileImpl {
            body:           quote! {
                fn reconcile<R: ::loro_surgeon::reconcile::Reconciler>(&self, r: R) -> Result<(), ::loro_surgeon::error::ReconcileError> {
                    self.0.reconcile(r)
                }
            },
            key_type:       quote! { <#inner_ty as ::loro_surgeon::reconcile::Reconcile>::Key },
            key_fn:         quote! {
                fn key(&self) -> ::loro_surgeon::reconcile::LoadKey<Self::Key> {
                    self.0.key()
                }
            },
            hydrate_key_fn: TokenStream::new(),
        };
    }

    let field_indices: Vec<_> = (0..fields.unnamed.len()).map(syn::Index::from).collect();

    ReconcileImpl {
        body:           quote! {
            fn reconcile<R: ::loro_surgeon::reconcile::Reconciler>(&self, r: R) -> Result<(), ::loro_surgeon::error::ReconcileError> {
                let mut l = ::loro_surgeon::reconcile::Reconciler::list(r)?;
                #(
                    l.insert(#field_indices, &self.#field_indices)?;
                )*
                Ok(())
            }
        },
        key_type:       quote! { ::loro_surgeon::reconcile::NoKey },
        key_fn:         TokenStream::new(),
        hydrate_key_fn: TokenStream::new(),
    }
}

fn derive_unit_struct() -> ReconcileImpl {
    ReconcileImpl {
        body:           quote! {
            fn reconcile<R: ::loro_surgeon::reconcile::Reconciler>(&self, r: R) -> Result<(), ::loro_surgeon::error::ReconcileError> {
                r.null()
            }
        },
        key_type:       quote! { ::loro_surgeon::reconcile::NoKey },
        key_fn:         TokenStream::new(),
        hydrate_key_fn: TokenStream::new(),
    }
}
