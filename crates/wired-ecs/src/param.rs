use std::any::type_name;

use crate::types::ParamType;

pub trait Param: Sized + 'static {
    fn param_key() -> String {
        type_name::<Self>().to_string()
    }
    fn param_type() -> Vec<ParamType>;

    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}
