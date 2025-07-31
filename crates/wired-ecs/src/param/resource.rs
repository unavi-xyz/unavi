use crate::types::{Param as WParam, ParamData};

use super::Param;

pub struct Resource;

impl Param for Resource {
    fn parse_param(_data: &mut std::vec::IntoIter<ParamData>) -> Self {
        Resource
    }
    fn register_param() -> Option<WParam> {
        todo!()
    }
}
