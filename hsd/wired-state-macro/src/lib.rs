//! `#[state]`: declares where each of a struct's fields lives.
//!
//! The four destinations of the space model are four lifetimes, and the
//! annotation states which one a field wants rather than how to get it. What
//! the expansion writes is the dirty-set flush the host API is shaped for:
//! one batch per tick, atomic, and nothing per key.

mod emit;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemStruct,
    parse_macro_input,
};

/// Generates accessors and a `sync` for a state struct.
///
/// Each field carries `#[state(session)]` or `#[state(document)]`; a field
/// with no annotation is ordinary memory and is left alone.
///
/// ```ignore
/// #[wired_prelude::state]
/// #[derive(Default)]
/// struct GateState {
///     #[state(session)]
///     link: Option<LinkState>,
/// }
/// ```
///
/// `state.set_link(..)` marks the field dirty; `state.sync(&prim)` flushes
/// what changed and adopts what other peers said. A `session` field lives for
/// the session and is replicated; a `document` field is committed after the
/// flush, which survives only on a client that holds the document's key —
/// that is the point of declaring the destination rather than the mechanism.
#[proc_macro_attribute]
pub fn state(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);

    match emit::expand(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => {
            let message = format!("{err}");
            quote! { ::core::compile_error!(#message); }.into()
        }
    }
}
