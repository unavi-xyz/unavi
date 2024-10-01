use std::cell::Cell;

use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefCountCell, RefResource},
    data::ScriptData,
};

use super::bindings::{
    types::Vec3,
    wired::physics::types::{HostRigidBody, RigidBodyType},
};

#[derive(Debug)]
pub struct RigidBody {
    angvel: Vec3,
    linvel: Vec3,
    pub rigid_body_type: RigidBodyType,
    ref_count: RefCountCell,
}

impl RigidBody {
    fn new(rigid_body_type: RigidBodyType) -> Self {
        Self {
            angvel: Vec3::default(),
            linvel: Vec3::default(),
            ref_count: RefCountCell::default(),
            rigid_body_type,
        }
    }

    /// Returns the equivalent Bevy component.
    pub fn component(&self) -> avian3d::prelude::RigidBody {
        match self.rigid_body_type {
            RigidBodyType::Dynamic => avian3d::prelude::RigidBody::Dynamic,
            RigidBodyType::Fixed => avian3d::prelude::RigidBody::Static,
            RigidBodyType::Kinematic => avian3d::prelude::RigidBody::Kinematic,
        }
    }
}

impl RefCount for RigidBody {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for RigidBody {}

impl HostRigidBody for ScriptData {
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
        RigidBody::handle_drop(rep, &mut self.table)?;
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

        let res = HostRigidBody::new(&mut data, RigidBodyType::Dynamic).unwrap();

        crate::api::utils::tests::test_drop(&mut data, res);
    }

    #[test]
    #[traced_test]
    fn test_new() {
        let (_, mut data) = init_test_data();

        let res = HostRigidBody::new(&mut data, RigidBodyType::Dynamic).unwrap();

        crate::api::utils::tests::test_new(&mut data, res);
    }
}
