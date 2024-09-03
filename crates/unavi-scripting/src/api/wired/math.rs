pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-math",
    });

    pub use self::wired::math::*;
}

impl From<bindings::types::Vec3> for bevy::prelude::Vec3 {
    fn from(val: bindings::types::Vec3) -> Self {
        bevy::prelude::Vec3::new(val.x, val.y, val.z)
    }
}

impl From<bevy::prelude::Vec3> for bindings::types::Vec3 {
    fn from(value: bevy::prelude::Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl Default for bindings::types::Vec3 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl From<bindings::types::Quat> for bevy::prelude::Quat {
    fn from(val: bindings::types::Quat) -> Self {
        bevy::prelude::Quat::from_xyzw(val.x, val.y, val.z, val.w)
    }
}

impl From<bevy::prelude::Quat> for bindings::types::Quat {
    fn from(value: bevy::prelude::Quat) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl From<bindings::types::Transform> for bevy::prelude::Transform {
    fn from(val: bindings::types::Transform) -> Self {
        bevy::prelude::Transform {
            rotation: val.rotation.into(),
            scale: val.scale.into(),
            translation: val.translation.into(),
        }
    }
}

impl From<bevy::prelude::Transform> for bindings::types::Transform {
    fn from(value: bevy::prelude::Transform) -> Self {
        Self {
            rotation: value.rotation.into(),
            scale: value.scale.into(),
            translation: value.translation.into(),
        }
    }
}
