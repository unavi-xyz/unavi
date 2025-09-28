use crate::types::{Param as WParam, ParamData};

use super::Param;

#[derive(Default)]
pub struct Local<T: Default>(pub T);

impl<T: Default> Param for Local<T> {
    fn parse_param(_data: &mut std::vec::IntoIter<ParamData>) -> Self {
        Local(T::default())
    }
    fn register_param() -> Option<WParam> {
        None
    }
}
