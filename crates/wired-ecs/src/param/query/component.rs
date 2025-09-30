use crate::Component;

pub trait QueriedComponent<'d> {
    type Component: Component;
    type Ref;
    type Mut;

    fn size() -> usize {
        Self::Component::size()
    }
    fn register() -> u32 {
        Self::Component::register()
    }

    fn from_bytes(bytes: &'d [u8]) -> Self::Ref;
    fn from_bytes_mut(bytes: &'d mut [u8]) -> Self::Mut;

    fn mutability() -> bool;
}

impl<'d, T: Component> QueriedComponent<'d> for &'d T {
    type Component = T;
    type Ref = &'d T;
    type Mut = &'d T;

    fn from_bytes(bytes: &'d [u8]) -> Self::Ref {
        T::view(bytes)
    }
    fn from_bytes_mut(bytes: &'d mut [u8]) -> Self::Mut {
        T::view(bytes)
    }
    fn mutability() -> bool {
        false
    }
}
impl<'d, T: Component> QueriedComponent<'d> for &'d mut T {
    type Component = T;
    type Ref = &'d T;
    type Mut = &'d mut T;

    fn from_bytes(bytes: &'d [u8]) -> Self::Ref {
        T::view(bytes)
    }
    fn from_bytes_mut(bytes: &'d mut [u8]) -> Self::Mut {
        T::view_mut(bytes)
    }
    fn mutability() -> bool {
        true
    }
}
