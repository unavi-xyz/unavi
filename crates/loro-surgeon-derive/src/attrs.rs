//! Attribute parsing for `#[loro(...)]` and `#[key]`.

use syn::{
    Attribute,
    Lit,
};

#[derive(Debug, Default)]
pub struct ContainerAttrs {
    /// Root key for `DocSync`: `#[loro(root = "key")]`.
    pub root: Option<String>,
}

/// How a field is stored in / read from Loro.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// Plain key/value entry — the default.
    #[default]
    Plain,
    /// Serialized to/from a JSON string.
    Json,
    /// Stored in a `LoroMovableList` instead of `LoroList`.
    Movable,
    /// Inlined into the parent map.
    Flatten,
}

#[derive(Debug, Default)]
pub struct FieldAttrs {
    pub is_key:           bool,
    pub strategy:         Strategy,
    pub rename:           Option<String>,
    pub missing:          Option<MissingStrategy>,
    pub with_module:      Option<String>,
    pub custom_hydrate:   Option<String>,
    pub custom_reconcile: Option<String>,
}

#[derive(Debug)]
pub enum MissingStrategy {
    Default,
    Function(String),
}

impl ContainerAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = Self::default();
        for attr in attrs {
            if !attr.path().is_ident("loro") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("root") {
                    let value = meta.value()?;
                    let lit: Lit = value.parse()?;
                    if let Lit::Str(s) = lit {
                        result.root = Some(s.value());
                    }
                }
                Ok(())
            })?;
        }
        Ok(result)
    }
}

impl FieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut result = Self::default();
        for attr in attrs {
            if attr.path().is_ident("key") {
                result.is_key = true;
                continue;
            }
            if !attr.path().is_ident("loro") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value = meta.value()?;
                    let lit: Lit = value.parse()?;
                    if let Lit::Str(s) = lit {
                        result.rename = Some(s.value());
                    }
                } else if meta.path.is_ident("json") {
                    result.strategy = Strategy::Json;
                } else if meta.path.is_ident("movable") {
                    result.strategy = Strategy::Movable;
                } else if meta.path.is_ident("flatten") {
                    result.strategy = Strategy::Flatten;
                } else if meta.path.is_ident("default") {
                    if meta.input.peek(syn::Token![=]) {
                        let value = meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(s) = lit {
                            result.missing = Some(MissingStrategy::Function(s.value()));
                        }
                    } else {
                        result.missing = Some(MissingStrategy::Default);
                    }
                } else if meta.path.is_ident("with") {
                    let value = meta.value()?;
                    let lit: Lit = value.parse()?;
                    if let Lit::Str(s) = lit {
                        result.with_module = Some(s.value());
                    }
                } else if meta.path.is_ident("hydrate") {
                    let value = meta.value()?;
                    let lit: Lit = value.parse()?;
                    if let Lit::Str(s) = lit {
                        result.custom_hydrate = Some(s.value());
                    }
                } else if meta.path.is_ident("reconcile") {
                    let value = meta.value()?;
                    let lit: Lit = value.parse()?;
                    if let Lit::Str(s) = lit {
                        result.custom_reconcile = Some(s.value());
                    }
                }
                Ok(())
            })?;
        }
        Ok(result)
    }

    pub fn loro_key(&self, field_name: &str) -> String {
        self.rename
            .clone()
            .unwrap_or_else(|| field_name.to_string())
    }
}
