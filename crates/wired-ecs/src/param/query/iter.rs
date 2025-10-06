use std::marker::PhantomData;

use super::component_group::ComponentGroup;

pub struct QueryIter<'d, T> {
    data: core::slice::Iter<'d, Vec<u8>>,
    ents: core::slice::Iter<'d, u64>,
    _t: PhantomData<T>,
}
impl<'d, T> QueryIter<'d, T> {
    pub fn new(data: core::slice::Iter<'d, Vec<u8>>, ents: core::slice::Iter<'d, u64>) -> Self {
        Self {
            data,
            ents,
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
        let ent = *self.ents.next().unwrap();
        self.data.next().map(|x| T::from_bytes(ent, x))
    }
}

pub struct QueryIterMut<'d, T> {
    data: core::slice::IterMut<'d, Vec<u8>>,
    ents: core::slice::Iter<'d, u64>,
    _t: PhantomData<T>,
}
impl<'d, T> QueryIterMut<'d, T> {
    pub fn new(data: core::slice::IterMut<'d, Vec<u8>>, ents: core::slice::Iter<'d, u64>) -> Self {
        Self {
            data,
            ents,
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
        let ent = *self.ents.next().unwrap();
        self.data.next().map(|x| T::from_bytes_mut(ent, x))
    }
}
