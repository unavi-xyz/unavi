use super::component::QueriedComponent;

pub trait ComponentGroup<'d> {
    type Ref;
    type Mut;

    fn register_components() -> Vec<u32>;
    fn component_sizes() -> Vec<usize>;
    fn mutability() -> Vec<bool>;

    fn from_bytes(id: u64, bytes: &'d [u8]) -> Self::Ref;
    fn from_bytes_mut(id: u64, bytes: &'d mut [u8]) -> Self::Mut;
}

impl<'d, A> ComponentGroup<'d> for A
where
    A: QueriedComponent<'d>,
{
    type Ref = A::Ref;
    type Mut = A::Mut;

    fn register_components() -> Vec<u32> {
        vec![A::register()].into_iter().filter_map(|x| x).collect()
    }
    fn component_sizes() -> Vec<usize> {
        vec![A::size()].into_iter().filter_map(|x| x).collect()
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability()]
            .into_iter()
            .filter_map(|x| x)
            .collect()
    }

    fn from_bytes(id: u64, bytes: &'d [u8]) -> Self::Ref {
        A::from_bytes(id, bytes)
    }
    fn from_bytes_mut(id: u64, bytes: &'d mut [u8]) -> Self::Mut {
        A::from_bytes_mut(id, bytes)
    }
}

impl<'d, A> ComponentGroup<'d> for (A,)
where
    A: QueriedComponent<'d>,
{
    type Ref = (A::Ref,);
    type Mut = (A::Mut,);

    fn register_components() -> Vec<u32> {
        vec![A::register()].into_iter().filter_map(|x| x).collect()
    }
    fn component_sizes() -> Vec<usize> {
        vec![A::size()].into_iter().filter_map(|x| x).collect()
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability()]
            .into_iter()
            .filter_map(|x| x)
            .collect()
    }

    fn from_bytes(id: u64, bytes: &'d [u8]) -> Self::Ref {
        (A::from_bytes(id, bytes),)
    }
    fn from_bytes_mut(id: u64, bytes: &'d mut [u8]) -> Self::Mut {
        (A::from_bytes_mut(id, bytes),)
    }
}

impl<'d, A, B> ComponentGroup<'d> for (A, B)
where
    A: QueriedComponent<'d>,
    B: QueriedComponent<'d>,
{
    type Ref = (A::Ref, B::Ref);
    type Mut = (A::Mut, B::Mut);

    fn register_components() -> Vec<u32> {
        vec![A::register(), B::register()]
            .into_iter()
            .filter_map(|x| x)
            .collect()
    }
    fn component_sizes() -> Vec<usize> {
        vec![A::size(), B::size()]
            .into_iter()
            .filter_map(|x| x)
            .collect()
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability(), B::mutability()]
            .into_iter()
            .filter_map(|x| x)
            .collect()
    }

    fn from_bytes(id: u64, bytes: &'d [u8]) -> Self::Ref {
        let xa = A::size().unwrap();

        let (ba, bb) = bytes.split_at(xa);

        let a = A::from_bytes(id, ba);
        let b = B::from_bytes(id, bb);

        (a, b)
    }
    fn from_bytes_mut(id: u64, bytes: &'d mut [u8]) -> Self::Mut {
        let xa = A::size().unwrap();

        let (ba, bb) = bytes.split_at_mut(xa);

        let a = A::from_bytes_mut(id, ba);
        let b = B::from_bytes_mut(id, bb);

        (a, b)
    }
}

impl<'d, A, B, C> ComponentGroup<'d> for (A, B, C)
where
    A: QueriedComponent<'d>,
    B: QueriedComponent<'d>,
    C: QueriedComponent<'d>,
{
    type Ref = (A::Ref, B::Ref, C::Ref);
    type Mut = (A::Mut, B::Mut, C::Mut);

    fn register_components() -> Vec<u32> {
        vec![A::register(), B::register(), C::register()]
            .into_iter()
            .filter_map(|x| x)
            .collect()
    }
    fn component_sizes() -> Vec<usize> {
        vec![A::size(), B::size(), C::size()]
            .into_iter()
            .filter_map(|x| x)
            .collect()
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability(), B::mutability(), C::mutability()]
            .into_iter()
            .filter_map(|x| x)
            .collect()
    }

    fn from_bytes(id: u64, bytes: &'d [u8]) -> Self::Ref {
        let xa = A::size().unwrap();
        let xb = B::size().unwrap();

        let (ba, bb) = bytes.split_at(xa);
        let (bb, bc) = bb.split_at(xb);

        let a = A::from_bytes(id, ba);
        let b = B::from_bytes(id, bb);
        let c = C::from_bytes(id, bc);

        (a, b, c)
    }
    fn from_bytes_mut(id: u64, bytes: &'d mut [u8]) -> Self::Mut {
        let xa = A::size().unwrap();
        let xb = B::size().unwrap();

        let (ba, bb) = bytes.split_at_mut(xa);
        let (bb, bc) = bb.split_at_mut(xb);

        let a = A::from_bytes_mut(id, ba);
        let b = B::from_bytes_mut(id, bb);
        let c = C::from_bytes_mut(id, bc);

        (a, b, c)
    }
}
