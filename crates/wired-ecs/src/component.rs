use std::any::type_name;

use crate::host_api::register_component;

pub trait Component: Sized {
    fn key() -> String {
        type_name::<Self>().to_string()
    }

    fn register() -> u32 {
        match register_component(&crate::types::Component { key: Self::key() }) {
            Ok(id) => id,
            Err(e) => panic!("Failed to register component: {e}"),
        }
    }

    fn from_bytes(bytes: Vec<u8>) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}
