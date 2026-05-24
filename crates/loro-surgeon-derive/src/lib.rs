//! Derive macros for loro-surgeon.

mod attrs;
mod hydrate;
mod reconcile;
mod type_util;

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(Hydrate, attributes(loro, key))]
pub fn derive_hydrate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let result = match &input.data {
        Data::Struct(data) => hydrate::struct_impl::derive_hydrate_struct(&input, data),
        Data::Enum(data) => Ok(hydrate::enum_impl::derive_hydrate_enum(&input, data)),
        Data::Union(_) => Err(syn::Error::new_spanned(
            &input,
            "Hydrate cannot be derived for unions",
        )),
    };
    match result {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Reconcile, attributes(loro, key))]
pub fn derive_reconcile(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let result = match &input.data {
        Data::Struct(data) => reconcile::struct_impl::derive_reconcile_struct(&input, data),
        Data::Enum(data) => Ok(reconcile::enum_impl::derive_reconcile_enum(&input, data)),
        Data::Union(_) => Err(syn::Error::new_spanned(
            &input,
            "Reconcile cannot be derived for unions",
        )),
    };
    match result {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
