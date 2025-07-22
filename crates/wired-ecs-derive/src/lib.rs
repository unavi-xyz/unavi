use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, GenericParam, Generics, parse_macro_input, parse_quote};

#[proc_macro_derive(Param)]
pub fn derive_param(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Add a bound `T: Param` to every type parameter T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate param types.
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("#[derive(Param)] only supports structs"),
    };

    let field_param_types = fields.iter().map(|f| {
        let ty = &f.ty;

        quote! {
            {
                use wired_ecs::types::{ParamType, Primitive};
                match std::any::type_name::<#ty>() {
                    "bool" => ParamType::Primitive(Primitive::Boolean),
                    "f32" => ParamType::Primitive(Primitive::Float32),
                    "f64" => ParamType::Primitive(Primitive::Float64),
                    "i8"  => ParamType::Primitive(Primitive::Int8),
                    "i16" => ParamType::Primitive(Primitive::Int16),
                    "i32" => ParamType::Primitive(Primitive::Int32),
                    "i64" => ParamType::Primitive(Primitive::Int64),
                    "u8"  => ParamType::Primitive(Primitive::Uint8),
                    "u16" => ParamType::Primitive(Primitive::Uint16),
                    "u32" => ParamType::Primitive(Primitive::Uint32),
                    "u64" => ParamType::Primitive(Primitive::Uint64),
                    "&str" | "alloc::string::String" => ParamType::Primitive(Primitive::Text),
                    other => panic!("Unsupported primitive type in #[derive(Param)]: {}", other),
                }
            }
        }
    });

    let expanded = quote! {
        unsafe impl #impl_generics wired_ecs::bytemuck::Zeroable for #name #ty_generics #where_clause {}
        unsafe impl #impl_generics wired_ecs::bytemuck::Pod for #name #ty_generics #where_clause {}

        impl #impl_generics wired_ecs::Param for #name #ty_generics #where_clause {
            fn param_key() -> String {
                let name = std::any::type_name::<Self>();
                let parts: Vec<&str> = name.split("::").collect();

                let namespace = parts[1];
                let path = parts[2..].join("/").to_lowercase();

                format!("{namespace}:{path}")
            }
            fn param_type() -> Vec<wired_ecs::types::ParamType> {
                vec![
                    #(#field_param_types),*
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
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(wired_ecs::types::Param));
        }
    }
    generics
}
