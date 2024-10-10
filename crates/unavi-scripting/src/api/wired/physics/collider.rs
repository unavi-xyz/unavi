use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use wasm_bridge::component::Resource;

use crate::data::ScriptData;

use super::bindings::{
    types::{ShapeCylinder, Vec3},
    wired::physics::types::{HostCollider, Shape},
};

#[derive(Debug, Clone)]
pub struct ColliderRes(Arc<RwLock<ColliderData>>);

#[derive(Debug)]
pub struct ColliderData {
    pub density: f32,
    pub shape: Shape,
}

impl ColliderData {
    pub fn new(shape: Shape) -> Self {
        Self {
            density: 1.0,
            shape,
        }
    }

    pub fn component(&self) -> avian3d::prelude::Collider {
        match self.shape {
            Shape::Cuboid(Vec3 { x, y, z }) => avian3d::prelude::Collider::cuboid(x, y, z),
            Shape::Cylinder(ShapeCylinder { height, radius }) => {
                avian3d::prelude::Collider::cylinder(radius, height)
            }
            Shape::Sphere(radius) => avian3d::prelude::Collider::sphere(radius),
        }
    }
}

impl ColliderRes {
    pub fn read(&self) -> RwLockReadGuard<ColliderData> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<ColliderData> {
        self.0.write().unwrap()
    }
}

impl HostCollider for ScriptData {
    fn new(&mut self, shape: Shape) -> wasm_bridge::Result<Resource<ColliderRes>> {
        let data = ColliderData::new(shape);
        let res = self.table.push(ColliderRes(Arc::new(RwLock::new(data))))?;
        Ok(res)
    }

    fn density(&mut self, self_: Resource<ColliderRes>) -> wasm_bridge::Result<f32> {
        let data = self.table.get(&self_)?.read();
        Ok(data.density)
    }
    fn set_density(&mut self, self_: Resource<ColliderRes>, value: f32) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.write();
        data.density = value;
        Ok(())
    }

    fn drop(&mut self, rep: Resource<ColliderRes>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
