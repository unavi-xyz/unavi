use crate::Component;

pub trait QueriedComponent<'d> {
    type Ref;
    type Mut;

    fn size() -> Option<usize>;
    fn register() -> Option<u32>;
    fn mutability() -> Option<bool>;

    fn from_bytes(id: u64, bytes: &'d [u8]) -> Self::Ref;
    fn from_bytes_mut(id: u64, bytes: &'d mut [u8]) -> Self::Mut;
}

impl<'d, T: Component> QueriedComponent<'d> for &'d T {
    type Ref = &'d T;
    type Mut = &'d T;

    fn size() -> Option<usize> {
        Some(T::size())
    }
    fn register() -> Option<u32> {
        Some(T::register())
    }
    fn mutability() -> Option<bool> {
        Some(false)
    }

    fn from_bytes(_: u64, bytes: &'d [u8]) -> Self::Ref {
        T::view(bytes)
    }
    fn from_bytes_mut(_: u64, bytes: &'d mut [u8]) -> Self::Mut {
        T::view(bytes)
    }
}
impl<'d, T: Component> QueriedComponent<'d> for &'d mut T {
    type Ref = &'d T;
    type Mut = &'d mut T;

    fn size() -> Option<usize> {
        Some(T::size())
    }
    fn register() -> Option<u32> {
        Some(T::register())
    }
    fn mutability() -> Option<bool> {
        Some(true)
    }

    fn from_bytes(_: u64, bytes: &'d [u8]) -> Self::Ref {
        T::view(bytes)
    }
    fn from_bytes_mut(_: u64, bytes: &'d mut [u8]) -> Self::Mut {
        T::view_mut(bytes)
    }
}
