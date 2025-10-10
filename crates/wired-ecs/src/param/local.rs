use std::{marker::PhantomData, ptr::NonNull};

use bytemuck::Pod;

use crate::{
    ParamState,
    types::{Param as WParam, ParamData},
};

use super::{Param, ParamMeta};

pub struct Local<T> {
    raw: NonNull<Vec<u8>>,
    _t: PhantomData<T>,
}

impl<T> AsRef<T> for Local<T>
where
    T: Pod,
{
    fn as_ref(&self) -> &T {
        bytemuck::from_bytes(unsafe { self.raw.as_ref() })
    }
}
impl<T> AsMut<T> for Local<T>
where
    T: Pod,
{
    fn as_mut(&mut self) -> &mut T {
        bytemuck::from_bytes_mut(unsafe { self.raw.as_mut() })
    }
}

impl<T> Param for Local<T>
where
    T: Pod + Default,
{
    fn register_param(states: &mut Vec<ParamState>) -> Option<WParam> {
        let inner = T::default();
        let raw = bytemuck::bytes_of(&inner).to_vec();
        states.push(ParamState { raw });
        None
    }
    fn meta() -> Option<ParamMeta> {
        None
    }
    fn parse_param(
        state: &mut std::slice::IterMut<ParamState>,
        _: &mut std::vec::IntoIter<ParamData>,
    ) -> Self {
        let p_state = state.next().expect("param state not found");
        Local {
            raw: (&mut p_state.raw).into(),
            _t: PhantomData,
        }
    }
}
