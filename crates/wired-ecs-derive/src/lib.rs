use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, GenericParam, Generics, parse_macro_input, parse_quote};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Add a bound `T: Component` to every type T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate component types.
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("#[derive(Component)] only supports structs"),
    };

    let field_component_types = fields.iter().map(|f| {
        let ty = &f.ty;

        quote! {
            {
                use wired_ecs::types::{ComponentType, Primitive};
                match std::any::type_name::<#ty>() {
                    "bool" => ComponentType::Primitive(Primitive::Boolean),
                    "f32" => ComponentType::Primitive(Primitive::Float32),
                    "f64" => ComponentType::Primitive(Primitive::Float64),
                    "i8"  => ComponentType::Primitive(Primitive::Int8),
                    "i16" => ComponentType::Primitive(Primitive::Int16),
                    "i32" => ComponentType::Primitive(Primitive::Int32),
                    "i64" => ComponentType::Primitive(Primitive::Int64),
                    "u8"  => ComponentType::Primitive(Primitive::Uint8),
                    "u16" => ComponentType::Primitive(Primitive::Uint16),
                    "u32" => ComponentType::Primitive(Primitive::Uint32),
                    "u64" => ComponentType::Primitive(Primitive::Uint64),
                    "&str" | "alloc::string::String" => ComponentType::Primitive(Primitive::Text),
                    other => panic!("Unsupported primitive type in #[derive(Component)]: {}", other),
                }
            }
        }
    });

    let expanded = quote! {
        unsafe impl #impl_generics wired_ecs::bytemuck::Zeroable for #name #ty_generics #where_clause {}
        unsafe impl #impl_generics wired_ecs::bytemuck::Pod for #name #ty_generics #where_clause {}

        impl #impl_generics wired_ecs::Component for #name #ty_generics #where_clause {
            fn key() -> String {
                let name = std::any::type_name::<Self>();
                let parts: Vec<&str> = name.split("::").collect();

                let namespace = parts[1];
                let path = parts[2..].join("/").to_lowercase();

                format!("{namespace}:{path}")
            }
            fn component_types() -> Vec<wired_ecs::types::ComponentType> {
                vec![
                    #(#field_component_types),*
                ]
            }
            fn from_bytes(bytes: &[u8]) -> Self {
                wired_ecs::bytemuck::from_bytes::<Self>(bytes).clone()
            }
            fn to_bytes(&self) -> Vec<u8> {
                wired_ecs::bytemuck::bytes_of(self).to_vec()
            }
        }
    };

    TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for component in &mut generics.params {
        if let GenericParam::Type(ref mut type_component) = *component {
            type_component
                .bounds
                .push(parse_quote!(wired_ecs::types::Component));
        }
    }
    generics
}
