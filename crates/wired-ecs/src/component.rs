use std::any::type_name;

use crate::{host_api::register_component, types::ComponentType};

pub trait Component: Sized + 'static {
    fn key() -> String {
        type_name::<Self>().to_string()
    }
    fn component_types() -> Vec<ComponentType>;

    fn register() -> u64 {
        match register_component(&crate::types::Component {
            key: Self::key(),
            types: Self::component_types(),
        }) {
            Ok(id) => id,
            Err(e) => panic!("Failed to register component: {e}"),
        }
    }

    /// Length of the component, in bytes.
    fn size() -> usize {
        std::mem::size_of::<Self>()
    }
    fn to_bytes(&self) -> Vec<u8>;

    fn view(bytes: &[u8]) -> &Self;
    fn view_mut(bytes: &mut [u8]) -> &mut Self;
}
