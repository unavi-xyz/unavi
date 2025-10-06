use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::quote;
use syn::{Data, DeriveInput, GenericParam, Generics, parse_macro_input, parse_quote};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let crate_ident = match crate_name("wired-ecs") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(#ident)
        }
        Err(_) => quote!(wired_ecs),
    };

    // Add a bound `T: Component` to every type T.
    let generics = add_trait_bounds(crate_ident.clone(), input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate component types.
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => return TokenStream::from(quote! {}),
        // _ => panic!("#[derive(Component)] only supports structs"),
    };

    let field_component_types = fields.iter().map(|f| {
        let ty = &f.ty;

        quote! {
            {
                use #crate_ident::types::{ComponentType, Primitive};
                match std::any::type_name::<#ty>() {
                    "bool" => ComponentType::Primitive(Primitive::Bool),
                    "f32" => ComponentType::Primitive(Primitive::F32),
                    "f64" => ComponentType::Primitive(Primitive::F64),
                    "i8"  => ComponentType::Primitive(Primitive::I8),
                    "i16" => ComponentType::Primitive(Primitive::I16),
                    "i32" => ComponentType::Primitive(Primitive::I32),
                    "i64" => ComponentType::Primitive(Primitive::I64),
                    "u8"  => ComponentType::Primitive(Primitive::U8),
                    "u16" => ComponentType::Primitive(Primitive::U16),
                    "u32" => ComponentType::Primitive(Primitive::U32),
                    "u64" => ComponentType::Primitive(Primitive::U64),
                    "usize" => ComponentType::Primitive(Primitive::U32),
                    "&str" | "alloc::string::String" => ComponentType::Primitive(Primitive::String),
                    other => panic!("Unsupported primitive type in #[derive(Component)]: {}", other),
                }
            }
        }
    });

    let expanded = quote! {
        unsafe impl #impl_generics #crate_ident::bytemuck::Zeroable for #name #ty_generics #where_clause {}
        unsafe impl #impl_generics #crate_ident::bytemuck::Pod for #name #ty_generics #where_clause {}

        impl #impl_generics #crate_ident::Component for #name #ty_generics #where_clause {
            fn key() -> String {
                let name = std::any::type_name::<Self>();
                let parts: Vec<&str> = name.split("::").collect();

                let namespace = parts[1];
                let path = parts[2..].join("/").to_lowercase();

                format!("{namespace}:{path}")
            }
            fn component_types() -> Vec<#crate_ident::types::ComponentType> {
                vec![
                    #(#field_component_types),*
                ]
            }
            fn to_bytes(&self) -> Vec<u8> {
                #crate_ident::bytemuck::bytes_of(self).to_vec()
            }
            fn view(bytes: &[u8]) -> &Self {
                #crate_ident::bytemuck::from_bytes(bytes)
            }
            fn view_mut(bytes: &mut [u8]) -> &mut Self {
                #crate_ident::bytemuck::from_bytes_mut(bytes)
            }
        }
    };

    TokenStream::from(expanded)
}

fn add_trait_bounds(crate_ident: proc_macro2::TokenStream, mut generics: Generics) -> Generics {
    for component in &mut generics.params {
        if let GenericParam::Type(ref mut type_component) = *component {
            type_component
                .bounds
                .push(parse_quote!(#crate_ident::types::Component));
        }
    }
    generics
}
