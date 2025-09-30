use std::{marker::PhantomData, ptr::NonNull};

use group::QueryGroup;
use iter::{QueryIter, QueryIterMut};

use crate::types::{Param as WParam, ParamData, Query as WQuery};

use super::Param;

mod component;
mod group;
mod iter;

/// Typed view of query data.
pub struct Query<T> {
    // This should be a safe reference, not a raw pointer, but despite my best
    // efforts I could not figure out the lifetimes to do so. ｡ﾟ･ (>﹏<) ･ﾟ｡
    raw: NonNull<Vec<Vec<u8>>>,
    _t: PhantomData<T>,
}

impl<T> Query<T> {
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

impl<T> Param for Query<T>
where
    T: QueryGroup<'static>,
{
    fn register_param() -> Option<WParam> {
        Some(WParam::Query(WQuery {
            components: T::register_components(),
            constraints: Vec::new(),
        }))
    }
    fn is_mutable() -> bool {
        T::is_mutable()
    }
    /// SAFETY:
    /// - Underlying data must remain alive through other means, Query holds a weak reference.
    fn parse_param(data: &mut std::slice::IterMut<ParamData>) -> Self {
        let ParamData::Query(raw) = data.next().unwrap();

        Self {
            raw: (&mut raw.data).into(),
            _t: PhantomData,
        }
    }
}
