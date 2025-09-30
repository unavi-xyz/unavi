use crate::types::{Param as WParam, ParamData};

use super::Param;

pub struct Resource;

impl Param for Resource {
    fn register_param() -> Option<WParam> {
        todo!()
    }
    fn is_mutable() -> bool {
        false
    }
    fn parse_param(_: &mut std::slice::IterMut<ParamData>) -> Self {
        Resource
    }
}
