use std::{marker::PhantomData, ptr::NonNull};

use component_group::ComponentGroup;
use constraint::Constraint;
use iter::{QueryIter, QueryIterMut};

use crate::{
    ParamState,
    types::{Param as WParam, ParamData, Query as WQuery},
};

use super::{Param, ParamMeta};

pub(crate) mod component;
pub(crate) mod component_group;
pub mod constraint;
mod iter;

/// Typed view of query data.
pub struct Query<T, U = ()> {
    // This should be a safe reference, not a raw pointer, but despite my best
    // efforts I could not figure out the lifetimes to do so. ｡ﾟ･ (>﹏<) ･ﾟ｡
    raw: NonNull<Vec<Vec<u8>>>,
    _t: PhantomData<T>,
    _u: PhantomData<U>,
}

impl<T, U> Query<T, U> {
    pub fn len(&self) -> usize {
        unsafe { self.raw.as_ref().len() }
    }
    pub fn is_empty(&self) -> bool {
        unsafe { self.raw.as_ref().is_empty() }
    }
    pub fn iter(&self) -> QueryIter<T> {
        unsafe { QueryIter::new(self.raw.as_ref().iter()) }
    }
    pub fn iter_mut(&mut self) -> QueryIterMut<T> {
        unsafe { QueryIterMut::new(self.raw.as_mut().iter_mut()) }
    }
}

impl<T, U> Param for Query<T, U>
where
    T: ComponentGroup<'static>,
    U: Constraint,
{
    fn register_param(_: &mut Vec<ParamState>) -> Option<WParam> {
        Some(WParam::Query(WQuery {
            components: T::register_components(),
            constraints: U::build_constraints(),
        }))
    }
    fn mutability() -> bool {
        T::mutability().iter().any(|x| *x)
    }
    fn meta() -> Option<ParamMeta> {
        Some(ParamMeta::Query {
            component_mut: T::mutability(),
            component_sizes: T::component_sizes(),
            constraints: U::concrete_constraints(),
        })
    }

    /// SAFETY:
    /// - Underlying data must remain alive through other means, Query holds a weak reference.
    /// - Data must remain effectively mutably owned by this Query.
    fn parse_param(
        _: &mut std::slice::IterMut<ParamState>,
        data: &mut std::slice::IterMut<ParamData>,
    ) -> Self {
        let ParamData::Query(raw) = data.next().unwrap();
        Self {
            raw: (&mut raw.data).into(),
            _t: PhantomData,
            _u: PhantomData,
        }
    }
}
