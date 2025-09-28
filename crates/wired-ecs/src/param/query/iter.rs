use std::marker::PhantomData;

use super::group::QueryGroup;

pub struct QueryIter<'a, T> {
    raw: core::slice::Iter<'a, Vec<u8>>,
    _t: PhantomData<T>,
}
impl<'a, T> QueryIter<'a, T> {
    pub fn new(raw: core::slice::Iter<'a, Vec<u8>>) -> Self {
        Self {
            raw,
            _t: PhantomData,
        }
    }
}
impl<'a, T: QueryGroup<'a>> Iterator for QueryIter<'a, T> {
    type Item = T::Ref;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|x| T::from_bytes(x))
    }
}

pub struct QueryIterMut<'a, T> {
    raw: core::slice::IterMut<'a, Vec<u8>>,
    _t: PhantomData<T>,
}
impl<'a, T> QueryIterMut<'a, T> {
    pub fn new(raw: core::slice::IterMut<'a, Vec<u8>>) -> Self {
        Self {
            raw,
            _t: PhantomData,
        }
    }
}
impl<'a, T: QueryGroup<'a>> Iterator for QueryIterMut<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|x| T::from_bytes_mut(x))
    }
}
