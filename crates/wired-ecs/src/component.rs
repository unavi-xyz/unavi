use std::any::type_name;

use crate::{host_api::register_component, types::ComponentType};

pub trait Component: Sized + 'static {
    fn key() -> String {
        type_name::<Self>().to_string()
    }
    fn component_types() -> Vec<ComponentType>;

    fn register() -> u64 {
        register_component(&crate::types::Component {
            key: Self::key(),
            types: Self::component_types(),
        })
    }

    fn byte_len() -> usize {
        std::mem::size_of::<Self>()
    }
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}
