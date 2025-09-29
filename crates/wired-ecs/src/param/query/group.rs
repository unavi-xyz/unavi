use super::component::QueriedComponent;

pub trait QueryGroup<'a> {
    type Ref;

    fn register_components() -> Vec<u32>;
    fn from_bytes(bytes: &'a [u8]) -> Self::Ref;
    fn from_bytes_mut(bytes: &'a mut [u8]) -> Self;
}

impl<'a, A> QueryGroup<'a> for A
where
    A: QueriedComponent<'a>,
{
    type Ref = &'a A::Component;

    fn register_components() -> Vec<u32> {
        vec![A::register()]
    }
    fn from_bytes(bytes: &'a [u8]) -> Self::Ref {
        A::from_bytes(bytes)
    }
    fn from_bytes_mut(bytes: &'a mut [u8]) -> Self {
        A::from_bytes_mut(bytes)
    }
}

impl<'a, A> QueryGroup<'a> for (A,)
where
    A: QueriedComponent<'a>,
{
    type Ref = (&'a A::Component,);

    fn register_components() -> Vec<u32> {
        vec![A::register()]
    }
    fn from_bytes(bytes: &'a [u8]) -> Self::Ref {
        (A::from_bytes(bytes),)
    }
    fn from_bytes_mut(bytes: &'a mut [u8]) -> Self {
        (A::from_bytes_mut(bytes),)
    }
}

impl<'a, A, B> QueryGroup<'a> for (A, B)
where
    A: QueriedComponent<'a>,
    B: QueriedComponent<'a>,
{
    type Ref = (&'a A::Component, &'a B::Component);

    fn register_components() -> Vec<u32> {
        vec![A::register(), B::register()]
    }
    fn from_bytes(bytes: &'a [u8]) -> Self::Ref {
        let xa = A::size();

        let (ba, bb) = bytes.split_at(xa);

        let a = A::from_bytes(ba);
        let b = B::from_bytes(bb);

        (a, b)
    }
    fn from_bytes_mut(bytes: &'a mut [u8]) -> Self {
        let xa = A::size();

        let (ba, bb) = bytes.split_at_mut(xa);

        let a = A::from_bytes_mut(ba);
        let b = B::from_bytes_mut(bb);

        (a, b)
    }
}

impl<'a, A, B, C> QueryGroup<'a> for (A, B, C)
where
    A: QueriedComponent<'a>,
    B: QueriedComponent<'a>,
    C: QueriedComponent<'a>,
{
    type Ref = (&'a A::Component, &'a B::Component, &'a C::Component);

    fn register_components() -> Vec<u32> {
        vec![A::register(), B::register(), C::register()]
    }
    fn from_bytes(bytes: &'a [u8]) -> Self::Ref {
        let xa = A::size();
        let xb = B::size();

        let (ba, bb) = bytes.split_at(xa);
        let (bb, bc) = bb.split_at(xb);

        let a = A::from_bytes(ba);
        let b = B::from_bytes(bb);
        let c = C::from_bytes(bc);

        (a, b, c)
    }
    fn from_bytes_mut(bytes: &'a mut [u8]) -> Self {
        let xa = A::size();
        let xb = B::size();

        let (ba, bb) = bytes.split_at_mut(xa);
        let (bb, bc) = bb.split_at_mut(xb);

        let a = A::from_bytes_mut(ba);
        let b = B::from_bytes_mut(bb);
        let c = C::from_bytes_mut(bc);

        (a, b, c)
    }
}
