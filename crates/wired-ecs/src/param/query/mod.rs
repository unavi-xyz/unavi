use std::marker::PhantomData;

use group::QueryGroup;
use iter::{QueryIter, QueryIterMut};

use crate::types::{Param as WParam, ParamData, Query as WQuery};

use super::Param;

mod component;
mod group;
mod iter;

pub struct Query<T> {
    raw: Vec<Vec<u8>>,
    _t: PhantomData<T>,
}
impl<T> Query<T> {
    pub fn len(&self) -> usize {
        self.raw.len()
    }
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
}
impl<T> Query<T> {
    pub fn iter(&self) -> QueryIter<T> {
        QueryIter::new(self.raw.iter())
    }
    pub fn iter_mut(&mut self) -> QueryIterMut<T> {
        QueryIterMut::new(self.raw.iter_mut())
    }
}

impl<'a, T> Param for Query<T>
where
    T: QueryGroup<'a>,
{
    fn parse_param(data: &mut std::vec::IntoIter<ParamData>) -> Self {
        let ParamData::Query(raw) = data.next().unwrap();
        Self {
            raw,
            _t: PhantomData,
        }
    }
    fn register_param() -> Option<WParam> {
        Some(WParam::Query(WQuery {
            components: T::register_components(),
            constraints: Vec::new(),
        }))
    }
}
