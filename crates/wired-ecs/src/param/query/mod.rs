use std::marker::PhantomData;

use component_group::ComponentGroup;
use constraint::Constraint;
use tuple_ref::{AsTupleMut, AsTupleRef};

use crate::{
    ParamState,
    types::{Param as WParam, ParamData, Query as WQuery},
};

use super::{Param, ParamMeta};

pub mod component;
pub mod component_group;
pub mod component_ref;
pub mod constraint;
pub mod owned_component;
pub mod tuple_ref;

pub struct Query<T, U = ()>
where
    T: ComponentGroup,
    U: Constraint,
{
    items: Vec<T::Owned>,
    _t: PhantomData<T>,
    _u: PhantomData<U>,
}

impl<T, U> Query<T, U>
where
    T: ComponentGroup,
    U: Constraint,
{
    #[must_use]
    pub const fn len(&self) -> usize {
        self.items.len()
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = <T::Owned as AsTupleRef<T::Ref>>::TRef<'_>> {
        self.items.iter().map(tuple_ref::AsTupleRef::as_tuple_ref)
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = <T::Owned as AsTupleMut<T::Mut>>::TMut<'_>> {
        self.items
            .iter_mut()
            .map(tuple_ref::AsTupleMut::as_tuple_mut)
    }
}

impl<T, U> Param for Query<T, U>
where
    T: ComponentGroup,
    U: Constraint,
{
    fn register_param(_: &mut Vec<ParamState>) -> Option<WParam> {
        Some(WParam::Query(WQuery {
            components: T::register_components(),
            constraints: U::build_constraints(),
        }))
    }
    fn meta() -> Option<ParamMeta> {
        Some(ParamMeta::Query {
            component_mut: T::mutability(),
            constraints: U::concrete_constraints(),
        })
    }

    fn parse_param(
        _: &mut std::slice::IterMut<ParamState>,
        data: &mut std::vec::IntoIter<ParamData>,
    ) -> Self {
        let ParamData::Query(data) = data.next().expect("missing param data");
        let items = data.into_iter().map(T::from_data).collect();
        Self {
            items,
            _t: PhantomData,
            _u: PhantomData,
        }
    }
}
