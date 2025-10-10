use crate::types::QueryData;

use super::{
    component::QueriedComponent,
    component_ref::{AsComponentMut, AsComponentRef},
    owned_component::OwnedComponent,
    tuple_ref::{AsTupleMut, AsTupleRef},
};

pub trait ComponentGroup {
    type Owned: AsTupleRef<Self::Ref> + AsTupleMut<Self::Mut>;
    type Ref;
    type Mut;

    fn register_components() -> Vec<u32>;
    fn mutability() -> Vec<bool>;

    fn from_data(data: QueryData) -> Self::Owned;
}

impl<A> ComponentGroup for A
where
    A: QueriedComponent,
    OwnedComponent<A::Owned, A::Ref, A::Mut>: AsComponentRef<A::Ref> + AsComponentMut<A::Mut>,
    OwnedComponent<A::Owned, A::Ref, A::Mut>: AsTupleRef<A::Ref> + AsTupleMut<A::Mut>,
{
    type Owned = OwnedComponent<A::Owned, A::Ref, A::Mut>;
    type Ref = A::Ref;
    type Mut = A::Mut;

    fn register_components() -> Vec<u32> {
        vec![A::register()].into_iter().flatten().collect()
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability()].into_iter().flatten().collect()
    }

    fn from_data(data: QueryData) -> Self::Owned {
        let mut iter = data.components.into_iter();
        A::from_bytes(data.entity, &mut iter)
    }
}

impl<A> ComponentGroup for (A,)
where
    A: QueriedComponent,
    OwnedComponent<A::Owned, A::Ref, A::Mut>: AsComponentRef<A::Ref> + AsComponentMut<A::Mut>,
    (OwnedComponent<A::Owned, A::Ref, A::Mut>,): AsTupleRef<(A::Ref,)> + AsTupleMut<(A::Mut,)>,
{
    type Owned = (OwnedComponent<A::Owned, A::Ref, A::Mut>,);
    type Ref = (A::Ref,);
    type Mut = (A::Mut,);

    fn register_components() -> Vec<u32> {
        vec![A::register()].into_iter().flatten().collect()
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability()].into_iter().flatten().collect()
    }

    fn from_data(data: QueryData) -> Self::Owned {
        let mut iter = data.components.into_iter();
        (A::from_bytes(data.entity, &mut iter),)
    }
}

impl<A, B> ComponentGroup for (A, B)
where
    A: QueriedComponent,
    OwnedComponent<A::Owned, A::Ref, A::Mut>: AsComponentRef<A::Ref> + AsComponentMut<A::Mut>,
    B: QueriedComponent,
    OwnedComponent<B::Owned, B::Ref, B::Mut>: AsComponentRef<B::Ref> + AsComponentMut<B::Mut>,
    (
        OwnedComponent<A::Owned, A::Ref, A::Mut>,
        OwnedComponent<B::Owned, B::Ref, B::Mut>,
    ): AsTupleRef<(A::Ref, B::Ref)> + AsTupleMut<(A::Mut, B::Mut)>,
{
    type Owned = (
        OwnedComponent<A::Owned, A::Ref, A::Mut>,
        OwnedComponent<B::Owned, B::Ref, B::Mut>,
    );
    type Ref = (A::Ref, B::Ref);
    type Mut = (A::Mut, B::Mut);

    fn register_components() -> Vec<u32> {
        vec![A::register(), B::register()]
            .into_iter()
            .flatten()
            .collect()
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability(), B::mutability()]
            .into_iter()
            .flatten()
            .collect()
    }

    fn from_data(data: QueryData) -> Self::Owned {
        let mut iter = data.components.into_iter();
        (
            A::from_bytes(data.entity, &mut iter),
            B::from_bytes(data.entity, &mut iter),
        )
    }
}

impl<A, B, C> ComponentGroup for (A, B, C)
where
    A: QueriedComponent,
    OwnedComponent<A::Owned, A::Ref, A::Mut>: AsComponentRef<A::Ref> + AsComponentMut<A::Mut>,
    B: QueriedComponent,
    OwnedComponent<B::Owned, B::Ref, B::Mut>: AsComponentRef<B::Ref> + AsComponentMut<B::Mut>,
    C: QueriedComponent,
    OwnedComponent<C::Owned, C::Ref, C::Mut>: AsComponentRef<C::Ref> + AsComponentMut<C::Mut>,
    (
        OwnedComponent<A::Owned, A::Ref, A::Mut>,
        OwnedComponent<B::Owned, B::Ref, B::Mut>,
        OwnedComponent<C::Owned, C::Ref, C::Mut>,
    ): AsTupleRef<(A::Ref, B::Ref, C::Ref)> + AsTupleMut<(A::Mut, B::Mut, C::Mut)>,
{
    type Owned = (
        OwnedComponent<A::Owned, A::Ref, A::Mut>,
        OwnedComponent<B::Owned, B::Ref, B::Mut>,
        OwnedComponent<C::Owned, C::Ref, C::Mut>,
    );
    type Ref = (A::Ref, B::Ref, C::Ref);
    type Mut = (A::Mut, B::Mut, C::Mut);

    fn register_components() -> Vec<u32> {
        vec![A::register(), B::register(), C::register()]
            .into_iter()
            .flatten()
            .collect()
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability(), B::mutability(), C::mutability()]
            .into_iter()
            .flatten()
            .collect()
    }

    fn from_data(data: QueryData) -> Self::Owned {
        let mut iter = data.components.into_iter();
        (
            A::from_bytes(data.entity, &mut iter),
            B::from_bytes(data.entity, &mut iter),
            C::from_bytes(data.entity, &mut iter),
        )
    }
}
