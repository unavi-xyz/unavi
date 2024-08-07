use std::cell::Cell;

use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefCountCell, RefResource},
    state::StoreState,
};

use super::wired::physics::types::{HostCollider, Shape};

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
}

impl HostCollider for StoreState {
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
    use bevy::prelude::*;
    use tracing_test::traced_test;

    use crate::api::wired_physics::wired::physics::types::Sphere;

    use super::*;

    #[test]
    #[traced_test]
    fn test_drop() {
        let mut world = World::default();
        let ent = world.spawn(()).id();
        let mut state = StoreState::new("test_drop".to_string(), ent);

        let shape = Shape::Sphere(Sphere { radius: 0.5 });
        let res = HostCollider::new(&mut state, shape).unwrap();

        crate::api::utils::tests::test_drop(&mut state, res);
    }

    #[test]
    #[traced_test]
    fn test_new() {
        let mut world = World::default();
        let ent = world.spawn(()).id();
        let mut state = StoreState::new("test_new".to_string(), ent);

        let shape = Shape::Sphere(Sphere { radius: 0.5 });
        let res = HostCollider::new(&mut state, shape).unwrap();

        crate::api::utils::tests::test_new(&mut state, res);
    }
}
