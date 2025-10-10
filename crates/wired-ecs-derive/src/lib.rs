use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::quote;
use syn::{DeriveInput, GenericParam, Generics, parse_macro_input, parse_quote};

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

    let expanded = quote! {
        impl #impl_generics #crate_ident::Component for #name #ty_generics #where_clause {
            fn key() -> String {
                let name = std::any::type_name::<Self>();
                let parts: Vec<&str> = name.split("::").collect();

                let namespace = parts[1];
                let path = parts[2..].join("/").to_lowercase();

                format!("{namespace}:{path}")
            }
            fn to_bytes(&self) -> Vec<u8> {
                #crate_ident::bincode::encode_to_vec(&self, #crate_ident::bincode::config::standard()).unwrap()
            }
            fn from_bytes(bytes: Vec<u8>) -> Self {
                let (val, _) = #crate_ident::bincode::decode_from_slice(&bytes, #crate_ident::bincode::config::standard()).unwrap();
                val
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
