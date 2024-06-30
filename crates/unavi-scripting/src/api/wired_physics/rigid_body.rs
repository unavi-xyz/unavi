use wasm_bridge::component::Resource;

use crate::state::StoreState;

use super::wired::{
    math::types::Vec3,
    physics::types::{HostRigidBody, RigidBodyType},
};

pub struct RigidBody {
    rigid_body_type: RigidBodyType,
}

impl RigidBody {
    fn new(rigid_body_type: RigidBodyType) -> Self {
        Self { rigid_body_type }
    }
}

impl HostRigidBody for StoreState {
    fn new(&mut self, rigid_body_type: RigidBodyType) -> wasm_bridge::Result<Resource<RigidBody>> {
        let rigid_body = RigidBody::new(rigid_body_type);

        let res = self.table.push(rigid_body)?;
        let rep = res.rep();

        Ok(Resource::new_own(rep))
    }

    fn angvel(&mut self, self_: Resource<RigidBody>) -> wasm_bridge::Result<Vec3> {
        todo!();
    }
    fn set_angvel(
        &mut self,
        self_: wasm_bridge::component::Resource<RigidBody>,
        value: Vec3,
    ) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn linvel(&mut self, self_: Resource<RigidBody>) -> wasm_bridge::Result<Vec3> {
        todo!();
    }
    fn set_linvel(&mut self, self_: Resource<RigidBody>, value: Vec3) -> wasm_bridge::Result<()> {
        todo!();
    }

    fn drop(&mut self, _rep: Resource<RigidBody>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
