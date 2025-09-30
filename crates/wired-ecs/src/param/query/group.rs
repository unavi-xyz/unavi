use super::component::QueriedComponent;

pub trait QueryGroup<'d> {
    type Ref;
    type Mut;

    fn from_bytes(bytes: &'d [u8]) -> Self::Ref;
    fn from_bytes_mut(bytes: &'d mut [u8]) -> Self::Mut;

    fn register_components() -> Vec<u32>;
    fn mutability() -> Vec<bool>;
    fn component_sizes() -> Vec<usize>;
}

impl<'d, A> QueryGroup<'d> for A
where
    A: QueriedComponent<'d>,
{
    type Ref = A::Ref;
    type Mut = A::Mut;

    fn from_bytes(bytes: &'d [u8]) -> Self::Ref {
        A::from_bytes(bytes)
    }
    fn from_bytes_mut(bytes: &'d mut [u8]) -> Self::Mut {
        A::from_bytes_mut(bytes)
    }

    fn register_components() -> Vec<u32> {
        vec![A::register()]
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability()]
    }
    fn component_sizes() -> Vec<usize> {
        vec![A::size()]
    }
}

impl<'d, A> QueryGroup<'d> for (A,)
where
    A: QueriedComponent<'d>,
{
    type Ref = (A::Ref,);
    type Mut = (A::Mut,);

    fn from_bytes(bytes: &'d [u8]) -> Self::Ref {
        (A::from_bytes(bytes),)
    }
    fn from_bytes_mut(bytes: &'d mut [u8]) -> Self::Mut {
        (A::from_bytes_mut(bytes),)
    }

    fn register_components() -> Vec<u32> {
        vec![A::register()]
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability()]
    }
    fn component_sizes() -> Vec<usize> {
        vec![A::size()]
    }
}

impl<'d, A, B> QueryGroup<'d> for (A, B)
where
    A: QueriedComponent<'d>,
    B: QueriedComponent<'d>,
{
    type Ref = (A::Ref, B::Ref);
    type Mut = (A::Mut, B::Mut);

    fn from_bytes(bytes: &'d [u8]) -> Self::Ref {
        let xa = A::size();

        let (ba, bb) = bytes.split_at(xa);

        let a = A::from_bytes(ba);
        let b = B::from_bytes(bb);

        (a, b)
    }
    fn from_bytes_mut(bytes: &'d mut [u8]) -> Self::Mut {
        let xa = A::size();

        let (ba, bb) = bytes.split_at_mut(xa);

        let a = A::from_bytes_mut(ba);
        let b = B::from_bytes_mut(bb);

        (a, b)
    }

    fn register_components() -> Vec<u32> {
        vec![A::register(), B::register()]
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability(), B::mutability()]
    }
    fn component_sizes() -> Vec<usize> {
        vec![A::size(), B::size()]
    }
}

impl<'d, A, B, C> QueryGroup<'d> for (A, B, C)
where
    A: QueriedComponent<'d>,
    B: QueriedComponent<'d>,
    C: QueriedComponent<'d>,
{
    type Ref = (A::Ref, B::Ref, C::Ref);
    type Mut = (A::Mut, B::Mut, C::Mut);

    fn from_bytes(bytes: &'d [u8]) -> Self::Ref {
        let xa = A::size();
        let xb = B::size();

        let (ba, bb) = bytes.split_at(xa);
        let (bb, bc) = bb.split_at(xb);

        let a = A::from_bytes(ba);
        let b = B::from_bytes(bb);
        let c = C::from_bytes(bc);

        (a, b, c)
    }
    fn from_bytes_mut(bytes: &'d mut [u8]) -> Self::Mut {
        let xa = A::size();
        let xb = B::size();

        let (ba, bb) = bytes.split_at_mut(xa);
        let (bb, bc) = bb.split_at_mut(xb);

        let a = A::from_bytes_mut(ba);
        let b = B::from_bytes_mut(bb);
        let c = C::from_bytes_mut(bc);

        (a, b, c)
    }

    fn register_components() -> Vec<u32> {
        vec![A::register(), B::register(), C::register()]
    }
    fn mutability() -> Vec<bool> {
        vec![A::mutability(), B::mutability(), C::mutability()]
    }
    fn component_sizes() -> Vec<usize> {
        vec![A::size(), B::size(), C::size()]
    }
}
