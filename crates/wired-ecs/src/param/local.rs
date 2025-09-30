use crate::types::{Param as WParam, ParamData};

use super::{Param, ParamMeta};

#[derive(Default)]
pub struct Local<T: Default>(pub T);

impl<T: Default> Param for Local<T> {
    fn register_param() -> Option<WParam> {
        None
    }
    fn mutability() -> bool {
        false
    }
    fn meta() -> Option<ParamMeta> {
        None
    }
    fn parse_param(_: &mut std::slice::IterMut<ParamData>) -> Self {
        Local(T::default())
    }
}
