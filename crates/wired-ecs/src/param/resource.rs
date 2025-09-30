use crate::types::{Param as WParam, ParamData};

use super::{Param, ParamMeta};

pub struct Resource;

impl Param for Resource {
    fn register_param() -> Option<WParam> {
        todo!()
    }
    fn mutability() -> bool {
        false
    }
    fn meta() -> Option<ParamMeta> {
        None
    }
    fn parse_param(_: &mut std::slice::IterMut<ParamData>) -> Self {
        Resource
    }
}
