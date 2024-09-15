use std::cell::Cell;

use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefCountCell, RefResource},
    data::StoreData,
};

use super::bindings::{
    types::{ShapeCylinder, Vec3},
    wired::physics::types::{HostCollider, Shape},
};

#[derive(Debug)]
pub struct Collider {
    pub density: f32,
    pub shape: Shape,
    ref_count: RefCountCell,
}

impl RefCount for Collider {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for Collider {}

impl Collider {
    pub fn new(shape: Shape) -> Self {
        Self {
            density: 1.0,
            shape,
            ref_count: RefCountCell::default(),
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

impl HostCollider for StoreData {
    fn new(&mut self, shape: Shape) -> wasm_bridge::Result<Resource<Collider>> {
        let collider = Collider::new(shape);
        let table_res = self.table.push(collider)?;
        let res = Collider::from_res(&table_res, &self.table)?;
        Ok(res)
    }

    fn density(&mut self, self_: Resource<Collider>) -> wasm_bridge::Result<f32> {
        let res = self.table.get(&self_)?;
        Ok(res.density)
    }
    fn set_density(&mut self, self_: Resource<Collider>, value: f32) -> wasm_bridge::Result<()> {
        let res = self.table.get_mut(&self_)?;
        res.density = value;
        Ok(())
    }

    fn drop(&mut self, rep: Resource<Collider>) -> wasm_bridge::Result<()> {
        Collider::handle_drop(rep, &mut self.table)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::api::utils::tests::init_test_data;

    use super::*;

    #[test]
    #[traced_test]
    fn test_drop() {
        let (_, mut data) = init_test_data();

        let shape = Shape::Sphere(0.5);
        let res = HostCollider::new(&mut data, shape).unwrap();

        crate::api::utils::tests::test_drop(&mut data, res);
    }

    #[test]
    #[traced_test]
    fn test_new() {
        let (_, mut data) = init_test_data();

        let shape = Shape::Sphere(0.5);
        let res = HostCollider::new(&mut data, shape).unwrap();

        crate::api::utils::tests::test_new(&mut data, res);
    }
}
