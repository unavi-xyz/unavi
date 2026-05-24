//! Shared type analysis utilities for derive macros.

/// Check if a type looks like `Vec<...>`.
pub fn is_vec(ty: &syn::Type) -> bool {
    extract_vec_inner_type(ty).is_some()
}

/// Extract the inner type from `Vec<T>`, returning `Some(T)`.
pub fn extract_vec_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return Some(inner);
    }
    None
}

/// Check if a type is `Option<...>`.
pub fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }
    false
}
