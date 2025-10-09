use crate::{
    ParamState,
    types::{Param as WParam, ParamData},
};

use super::{Param, ParamMeta};

pub struct Local<T> {
    inner: T,
}

impl<T> AsRef<T> for Local<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}
impl<T> AsMut<T> for Local<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Param for Local<T>
where
    T: Default,
{
    fn register_param(states: &mut Vec<ParamState>) -> Option<WParam> {
        // let inner = T::default();
        // let raw = bytemuck::bytes_of(&inner).to_vec();
        // states.push(ParamState { raw });

        None
    }
    fn mutability() -> bool {
        false
    }
    fn meta() -> Option<ParamMeta> {
        None
    }
    fn parse_param(
        state: &mut std::slice::IterMut<ParamState>,
        _: &mut std::vec::IntoIter<ParamData>,
    ) -> Self {
        // let p_state = state.next().expect("param state not found");
        Local {
            inner: T::default(),
        }
        // todo!()
    }
}
