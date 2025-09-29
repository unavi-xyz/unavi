use crate::Component;

pub trait QueriedComponent<'a> {
    type Component: Component;

    fn size() -> usize {
        Self::Component::size()
    }
    fn register() -> u32 {
        Self::Component::register()
    }

    fn from_bytes(bytes: &'a [u8]) -> &'a Self::Component;
    fn from_bytes_mut(bytes: &'a mut [u8]) -> Self;
}

impl<'a, T: Component> QueriedComponent<'a> for &'a T {
    type Component = T;

    fn from_bytes(bytes: &'a [u8]) -> &'a T {
        T::view(bytes)
    }
    fn from_bytes_mut(bytes: &'a mut [u8]) -> Self {
        T::view(bytes)
    }
}
impl<'a, T: Component> QueriedComponent<'a> for &'a mut T {
    type Component = T;

    fn from_bytes(bytes: &'a [u8]) -> &'a T {
        T::view(bytes)
    }
    fn from_bytes_mut(bytes: &'a mut [u8]) -> Self {
        T::view_mut(bytes)
    }
}
