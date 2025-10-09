use std::any::type_name;

use crate::{host_api::register_component, types::ComponentType};

pub trait Component: Sized {
    fn key() -> String {
        type_name::<Self>().to_string()
    }
    fn component_types() -> Vec<ComponentType>;

    fn register() -> u32 {
        match register_component(&crate::types::Component {
            key: Self::key(),
            types: Self::component_types(),
        }) {
            Ok(id) => id,
            Err(e) => panic!("Failed to register component: {e}"),
        }
    }

    fn from_bytes(bytes: Vec<u8>) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}
