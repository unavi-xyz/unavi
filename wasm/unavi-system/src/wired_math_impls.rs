use crate::bindings::wired::math::types::{Quat, Transform, Vec3};

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

impl PartialEq for Quat {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}
