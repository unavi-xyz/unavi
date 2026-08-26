//! Declares the configuration a crate's `secretspec.toml` describes, as a type
//! carrying the values its profile named.
//!
//! Reading the manifest during expansion keeps `secretspec` a host dependency
//! of this macro that never reaches the binary.

mod emit;
mod merge;

use std::{
    env,
    path::{
        Path,
        PathBuf,
    },
};

use anyhow::{
    Context,
    Result,
    bail,
};
use proc_macro::TokenStream;
use quote::quote;
use secretspec::{
    Config,
    codegen::build_ir,
};
use syn::{
    LitStr,
    parse_macro_input,
};

const DEFAULT_PROFILE: &str = "default";
const PROFILE_VAR: &str = "SECRETSPEC_PROFILE";

/// Declares a `Secrets` type for the manifest at `input`, a path relative to
/// the calling crate's root, under the profile [`PROFILE_VAR`] names.
///
/// Each secret reads from the environment the binary runs under, falling back
/// on the value its profile declared.
#[proc_macro]
pub fn declare(input: TokenStream) -> TokenStream {
    let manifest = parse_macro_input!(input as LitStr).value();

    match expand(&manifest) {
        Ok(tokens) => tokens.into(),
        Err(err) => {
            let message = format!("{err:#}");
            quote! { ::core::compile_error!(#message); }.into()
        }
    }
}

fn expand(manifest: &str) -> Result<proc_macro2::TokenStream> {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR")?);
    let manifest = root.join(manifest);

    let config = Config::try_from(manifest.as_path())
        .with_context(|| format!("read {}", manifest.display()))?;

    let profile = env::var(PROFILE_VAR).unwrap_or_else(|_| DEFAULT_PROFILE.to_owned());
    let ir = build_ir(&config);

    let Some(selected) = ir.profile_fields.iter().find(|it| it.name == profile) else {
        bail!(
            "'{profile}' is not defined in {}. Available profiles: {}",
            manifest.display(),
            ir.profiles.join(", ")
        );
    };

    let defaults = merge::defaults(&config, &profile, DEFAULT_PROFILE)?;
    let accessor = emit::accessor(&profile, &selected.fields, &defaults)?;
    let tracked = tracked(&manifest)?;

    Ok(quote! {
        #tracked
        #accessor
    })
}

/// Cargo cannot see what a macro read, so the expansion restates its inputs as
/// the two forms rustc does record: an included file and a compile-time
/// environment lookup. Without them an edited manifest would leave a stale
/// expansion in place.
fn tracked(manifest: &Path) -> Result<proc_macro2::TokenStream> {
    let manifest = manifest
        .to_str()
        .context("manifest path is not valid utf-8")?;

    Ok(quote! {
        const _: &::core::primitive::str = ::core::include_str!(#manifest);
        const _: ::core::option::Option<&::core::primitive::str> =
            ::core::option_env!(#PROFILE_VAR);
    })
}
