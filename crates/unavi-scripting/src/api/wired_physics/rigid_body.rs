use std::cell::Cell;

use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefResource},
    state::StoreState,
};

use super::wired::{
    math::types::Vec3,
    physics::types::{HostRigidBody, RigidBodyType},
};

pub struct RigidBody {
    angvel: bevy::math::Vec3,
    linvel: bevy::math::Vec3,
    pub rigid_body_type: RigidBodyType,
    ref_count: Cell<usize>,
}

impl RigidBody {
    fn new(rigid_body_type: RigidBodyType) -> Self {
        Self {
            angvel: bevy::math::Vec3::default(),
            linvel: bevy::math::Vec3::default(),
            ref_count: Cell::new(1),
            rigid_body_type,
        }
    }
}

impl RefCount for RigidBody {
    fn ref_count<'a>(&'a self) -> &'a Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for RigidBody {}

impl HostRigidBody for StoreState {
    fn new(&mut self, rigid_body_type: RigidBodyType) -> wasm_bridge::Result<Resource<RigidBody>> {
        let rigid_body = RigidBody::new(rigid_body_type);
        let table_res = self.table.push(rigid_body)?;
        let res = RigidBody::from_res(&table_res, &self.table)?;
        Ok(res)
    }

    fn angvel(&mut self, self_: Resource<RigidBody>) -> wasm_bridge::Result<Vec3> {
        let data = self.table.get(&self_)?;
        Ok(Vec3 {
            x: data.angvel.x,
            y: data.angvel.y,
            z: data.angvel.z,
        })
    }
    fn set_angvel(
        &mut self,
        self_: wasm_bridge::component::Resource<RigidBody>,
        value: Vec3,
    ) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.angvel.x = value.x;
        data.angvel.y = value.y;
        data.angvel.z = value.z;
        Ok(())
    }

    fn linvel(&mut self, self_: Resource<RigidBody>) -> wasm_bridge::Result<Vec3> {
        let data = self.table.get(&self_)?;
        Ok(Vec3 {
            x: data.linvel.x,
            y: data.linvel.y,
            z: data.linvel.z,
        })
    }
    fn set_linvel(&mut self, self_: Resource<RigidBody>, value: Vec3) -> wasm_bridge::Result<()> {
        let data = self.table.get_mut(&self_)?;
        data.linvel.x = value.x;
        data.linvel.y = value.y;
        data.linvel.z = value.z;
        Ok(())
    }

    fn drop(&mut self, rep: Resource<RigidBody>) -> wasm_bridge::Result<()> {
        RigidBody::handle_drop(rep, &mut self.table)
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_drop() {
        let (mut state, _) = StoreState::new("test_drop".to_string());

        let res = HostRigidBody::new(&mut state, RigidBodyType::Dynamic).unwrap();

        crate::api::utils::tests::test_drop(&mut state, res);
    }
}
