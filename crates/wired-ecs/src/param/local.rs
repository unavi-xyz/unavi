use crate::types::{Param as WParam, ParamData};

use super::Param;

#[derive(Default)]
pub struct Local<T: Default>(pub T);

impl<T: Default> Param for Local<T> {
    fn register_param() -> Option<WParam> {
        None
    }
    fn is_mutable() -> bool {
        false
    }
    fn parse_param(_: &mut std::slice::IterMut<ParamData>) -> Self {
        Local(T::default())
    }
}
