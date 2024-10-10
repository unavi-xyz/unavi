use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use wasm_bridge::component::Resource;

use crate::data::ScriptData;

use super::bindings::{
    types::Vec3,
    wired::physics::types::{HostRigidBody, RigidBodyType},
};

#[derive(Debug, Clone)]
pub struct RigidBodyRes(Arc<RwLock<RigidBodyData>>);

#[derive(Debug)]
pub struct RigidBodyData {
    angvel: Vec3,
    linvel: Vec3,
    pub rigid_body_type: RigidBodyType,
}

impl RigidBodyData {
    fn new(rigid_body_type: RigidBodyType) -> Self {
        Self {
            angvel: Vec3::default(),
            linvel: Vec3::default(),
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

impl RigidBodyRes {
    pub fn read(&self) -> RwLockReadGuard<RigidBodyData> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<RigidBodyData> {
        self.0.write().unwrap()
    }
}

impl HostRigidBody for ScriptData {
    fn new(
        &mut self,
        rigid_body_type: RigidBodyType,
    ) -> wasm_bridge::Result<Resource<RigidBodyRes>> {
        let data = RigidBodyRes(Arc::new(RwLock::new(RigidBodyData::new(rigid_body_type))));
        let res = self.table.push(data)?;
        Ok(res)
    }

    fn angvel(&mut self, self_: Resource<RigidBodyRes>) -> wasm_bridge::Result<Vec3> {
        let data = self.table.get(&self_)?.read();
        Ok(Vec3 {
            x: data.angvel.x,
            y: data.angvel.y,
            z: data.angvel.z,
        })
    }
    fn set_angvel(
        &mut self,
        self_: wasm_bridge::component::Resource<RigidBodyRes>,
        value: Vec3,
    ) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.write();
        data.angvel.x = value.x;
        data.angvel.y = value.y;
        data.angvel.z = value.z;
        Ok(())
    }

    fn linvel(&mut self, self_: Resource<RigidBodyRes>) -> wasm_bridge::Result<Vec3> {
        let data = self.table.get(&self_)?.read();
        Ok(Vec3 {
            x: data.linvel.x,
            y: data.linvel.y,
            z: data.linvel.z,
        })
    }
    fn set_linvel(
        &mut self,
        self_: Resource<RigidBodyRes>,
        value: Vec3,
    ) -> wasm_bridge::Result<()> {
        let mut data = self.table.get(&self_)?.write();
        data.linvel.x = value.x;
        data.linvel.y = value.y;
        data.linvel.z = value.z;
        Ok(())
    }

    fn drop(&mut self, rep: Resource<RigidBodyRes>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
