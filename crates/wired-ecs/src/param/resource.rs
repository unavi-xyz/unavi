use std::{marker::PhantomData, ptr::NonNull};

use crate::types::{Param as WParam, ParamData, Query as WQuery};

use super::{Param, ParamMeta, component_group::ComponentGroup};

pub struct Res<T>
where
    for<'a> &'a T: ComponentGroup<'a>,
{
    raw: NonNull<Vec<u8>>,
    _t: PhantomData<T>,
}
pub struct ResMut<T>
where
    for<'a> &'a T: ComponentGroup<'a>,
    for<'a> &'a mut T: ComponentGroup<'a>,
{
    raw: NonNull<Vec<u8>>,
    _t: PhantomData<T>,
}

impl<T> AsRef<T> for Res<T>
where
    for<'a> &'a T: ComponentGroup<'a, Ref = &'a T>,
{
    fn as_ref(&self) -> &T {
        let data = unsafe { self.raw.as_ref() };
        <&T>::from_bytes(data)
    }
}
impl<T> AsRef<T> for ResMut<T>
where
    for<'a> &'a T: ComponentGroup<'a, Ref = &'a T>,
    for<'a> &'a mut T: ComponentGroup<'a>,
{
    fn as_ref(&self) -> &T {
        let data = unsafe { self.raw.as_ref() };
        <&T>::from_bytes(data)
    }
}

impl<T> AsMut<T> for ResMut<T>
where
    for<'a> &'a T: ComponentGroup<'a>,
    for<'a> &'a mut T: ComponentGroup<'a, Mut = &'a mut T>,
{
    fn as_mut(&mut self) -> &mut T {
        let data = unsafe { self.raw.as_mut() };
        <&mut T>::from_bytes_mut(data)
    }
}

impl<T> Param for Res<T>
where
    for<'a> &'a T: ComponentGroup<'a>,
{
    fn register_param() -> Option<WParam> {
        Some(WParam::Query(WQuery {
            components: <&T>::register_components(),
            constraints: Vec::new(),
        }))
    }
    fn mutability() -> bool {
        false
    }
    fn meta() -> Option<ParamMeta> {
        Some(ParamMeta::Query {
            component_mut: <&T>::mutability(),
            component_sizes: <&T>::component_sizes(),
            constraints: Vec::new(),
        })
    }
    fn parse_param(data: &mut std::slice::IterMut<ParamData>) -> Self {
        let ParamData::Query(q) = data.next().unwrap();
        let c_data = q.data.get_mut(0).expect("resource should always exist");
        Self {
            raw: c_data.into(),
            _t: PhantomData,
        }
    }
}
impl<T> Param for ResMut<T>
where
    for<'a> &'a T: ComponentGroup<'a>,
    for<'a> &'a mut T: ComponentGroup<'a>,
{
    fn register_param() -> Option<WParam> {
        Some(WParam::Query(WQuery {
            components: <&mut T>::register_components(),
            constraints: Vec::new(),
        }))
    }
    fn mutability() -> bool {
        true
    }
    fn meta() -> Option<ParamMeta> {
        Some(ParamMeta::Query {
            component_mut: <&mut T>::mutability(),
            component_sizes: <&mut T>::component_sizes(),
            constraints: Vec::new(),
        })
    }
    fn parse_param(data: &mut std::slice::IterMut<ParamData>) -> Self {
        let ParamData::Query(q) = data.next().unwrap();
        let c_data = q.data.get_mut(0).expect("resource should always exist");
        Self {
            raw: c_data.into(),
            _t: PhantomData,
        }
    }
}
