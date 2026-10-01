//! Renders the `Config` type a manifest describes.

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
/// A field the manifest gives a value is public by declaration: it becomes a
/// [`String`] carrying that value, overridable through the build environment
/// and then the runtime one. A field with no declared value is never compiled
/// in, only read at runtime, and fails the load if it is required and unset.
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
                "field '{}' is `as_path`, which only a runtime resolve can materialize",
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
        pub struct Config {
            #(#declarations)*
        }

        /// A required field with no declared value was unset at runtime.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct MissingConfig(pub &'static str);

        impl ::core::fmt::Display for MissingConfig {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(f, "required configuration `{}` is not set", self.0)
            }
        }

        impl ::core::error::Error for MissingConfig {}

        impl Config {
            /// Reads each field from the environment, falling back on the
            /// value its profile declared.
            pub fn load() -> ::core::result::Result<Self, MissingConfig> {
                ::core::result::Result::Ok(Self {
                    #(#initializers)*
                })
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

    let ty = if defaults.contains_key(&field.name) || !field.optional {
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
        || {
            if field.optional {
                quote! { ::std::env::var(#name).ok() }
            } else {
                quote! { ::std::env::var(#name).map_err(|_| MissingConfig(#name))? }
            }
        },
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
        .with_context(|| format!("field '{name}' does not name a Rust field"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str) -> IrField {
        IrField {
            name:        name.to_owned(),
            optional:    true,
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

    #[test]
    fn an_undeclared_value_is_read_only_at_runtime() {
        let rendered = render(&[field("TOKEN")], &BTreeMap::new());

        assert!(rendered.contains("pub token: Option<String>,"));
        assert!(rendered.contains(r#"env::var("TOKEN").ok()"#));
        assert!(!rendered.contains("option_env!"));
    }

    #[test]
    fn a_required_value_fails_the_load_when_unset() {
        let mut required = field("TOKEN");
        required.optional = false;
        let rendered = render(&[required], &BTreeMap::new());

        assert!(rendered.contains("pub token: String,"));
        assert!(rendered.contains(r#"map_err(|_| MissingConfig("TOKEN"))?"#));
        assert!(!rendered.contains("option_env!"));
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
