use wasm_bridge::component::Resource;

use crate::state::StoreState;

use super::wired::physics::types::{HostCollider, Shape};

pub struct Collider {
    density: f32,
    shape: Shape,
}

impl Collider {
    fn new(shape: Shape) -> Self {
        Self {
            density: 1.0,
            shape,
        }
    }
}

impl HostCollider for StoreState {
    fn new(&mut self, shape: Shape) -> wasm_bridge::Result<Resource<Collider>> {
        let collider = Collider::new(shape);

        let res = self.table.push(collider)?;
        let rep = res.rep();

        Ok(Resource::new_own(rep))
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

    fn drop(&mut self, _rep: Resource<Collider>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
