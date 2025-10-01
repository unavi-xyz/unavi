use std::marker::PhantomData;

use super::component_group::ComponentGroup;

pub struct QueryIter<'d, T> {
    raw: core::slice::Iter<'d, Vec<u8>>,
    _t: PhantomData<T>,
}
impl<'d, T> QueryIter<'d, T> {
    pub fn new(raw: core::slice::Iter<'d, Vec<u8>>) -> Self {
        Self {
            raw,
            _t: PhantomData,
        }
    }
}
impl<'d, T> Iterator for QueryIter<'d, T>
where
    T: ComponentGroup<'d>,
{
    type Item = T::Ref;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|x| T::from_bytes(x))
    }
}

pub struct QueryIterMut<'d, T> {
    raw: core::slice::IterMut<'d, Vec<u8>>,
    _t: PhantomData<T>,
}
impl<'d, T> QueryIterMut<'d, T> {
    pub fn new(raw: core::slice::IterMut<'d, Vec<u8>>) -> Self {
        Self {
            raw,
            _t: PhantomData,
        }
    }
}
impl<'d, T> Iterator for QueryIterMut<'d, T>
where
    T: ComponentGroup<'d>,
{
    type Item = T::Mut;
    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next().map(|x| T::from_bytes_mut(x))
    }
}
