use crate::types::QueryData;

use super::{
    component::{OwnedComponent, QueriedComponent}, component_ref::{AsComponentMut, AsComponentRef}, tuple_ref::{ AsTupleMut, AsTupleRef}
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
        A::from_bytes(data.entity, iter.next().unwrap())
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
        (A::from_bytes(data.entity, iter.next().unwrap()),)
    }
}

// impl<A, B> ComponentGroup for (A, B)
// where
//     A: QueriedComponent,
//     B: QueriedComponent,
// {
//     type Owned = (
//         OwnedComponent<A::Owned, A::Ref, A::Mut>,
//         OwnedComponent<B::Owned, B::Ref, B::Mut>,
//     );
//     type Ref = (A::Ref, B::Ref);
//     type Mut = (A::Mut, B::Mut);
//
//     fn register_components() -> Vec<u32> {
//         vec![A::register(), B::register()]
//             .into_iter()
//             .flatten()
//             .collect()
//     }
//     fn mutability() -> Vec<bool> {
//         vec![A::mutability(), B::mutability()]
//             .into_iter()
//             .flatten()
//             .collect()
//     }
//
//     fn from_data(data: QueryData) -> Self::Owned {
//         let mut iter = data.components.into_iter();
//         (
//             A::from_bytes(data.entity, iter.next().unwrap()),
//             B::from_bytes(data.entity, iter.next().unwrap()),
//         )
//     }
// }

// impl<A, B, C> ComponentGroup for (A, B, C)
// where
//     A: QueriedComponent,
//     B: QueriedComponent,
//     C: QueriedComponent,
// {
//     fn register_components() -> Vec<u32> {
//         vec![A::register(), B::register(), C::register()]
//             .into_iter()
//             .filter_map(|x| x)
//             .collect()
//     }
//     fn mutability() -> Vec<bool> {
//         vec![A::mutability(), B::mutability(), C::mutability()]
//             .into_iter()
//             .filter_map(|x| x)
//             .collect()
//     }
//
//     fn from_data(data: QueryData) -> Self {
//         let mut iter = data.components.into_iter();
//         (
//             A::from_bytes(data.entity, iter.next().unwrap()),
//             B::from_bytes(data.entity, iter.next().unwrap()),
//             C::from_bytes(data.entity, iter.next().unwrap()),
//         )
//     }
// }
