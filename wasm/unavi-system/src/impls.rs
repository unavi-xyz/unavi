use crate::bindings::wired::{
    gltf::{
        material::Material,
        mesh::{Mesh, Primitive},
        node::{Node, Transform},
    },
    math::types::{Vec3, Vec4},
};

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for Mesh {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, other: &Self) -> bool {
        (self.x == other.x) && (self.y == other.y) && (self.z == other.z)
    }
}
impl Default for Vec3 {
    fn default() -> Self {
        Self::splat(0.0)
    }
}
impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn splat(value: f32) -> Self {
        Self::new(value, value, value)
    }
}

impl PartialEq for Vec4 {
    fn eq(&self, other: &Self) -> bool {
        (self.x == other.x) && (self.y == other.y) && (self.z == other.z) && (self.w == other.w)
    }
}
impl Default for Vec4 {
    fn default() -> Self {
        Self::splat(0.0)
    }
}
impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn splat(value: f32) -> Self {
        Self::new(value, value, value, value)
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        (self.translation == other.translation)
            && (self.rotation == other.rotation)
            && (self.scale == other.scale)
    }
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::default(),
            rotation: Vec4::new(0.0, 0.0, 0.0, 1.0),
            scale: Vec3::splat(1.0),
        }
    }
}
