use std::cell::Cell;

use bevy::math::Quat;
use wasm_bridge::component::Resource;

use crate::StoreState;

use super::wired::math::types::HostQuat;

pub struct QuatRes {
    pub data: Quat,
    pub dirty: Cell<bool>,
    ref_count: usize,
}

impl QuatRes {
    pub fn new(data: Quat) -> Self {
        Self {
            data,
            dirty: Cell::new(true),
            ref_count: 1,
        }
    }
}

impl HostQuat for StoreState {
    fn new(&mut self, x: f32, y: f32, z: f32, w: f32) -> wasm_bridge::Result<Resource<QuatRes>> {
        let res = self.table.push(QuatRes::new(Quat::from_xyzw(x, y, z, w)))?;
        Ok(res)
    }
    fn default(&mut self) -> wasm_bridge::Result<Resource<QuatRes>> {
        let res = self.table.push(QuatRes::new(Quat::default()))?;
        Ok(res)
    }
    fn from_rotation_y(&mut self, angle: f32) -> wasm_bridge::Result<Resource<QuatRes>> {
        let res = self
            .table
            .push(QuatRes::new(Quat::from_rotation_y(angle)))?;
        Ok(res)
    }

    fn xyzw(&mut self, self_: Resource<QuatRes>) -> wasm_bridge::Result<(f32, f32, f32, f32)> {
        let quat = self.table.get(&self_)?;
        Ok((quat.data.x, quat.data.y, quat.data.z, quat.data.w))
    }
    fn x(&mut self, self_: Resource<QuatRes>) -> wasm_bridge::Result<f32> {
        let quat = self.table.get(&self_)?;
        Ok(quat.data.x)
    }
    fn y(&mut self, self_: Resource<QuatRes>) -> wasm_bridge::Result<f32> {
        let quat = self.table.get(&self_)?;
        Ok(quat.data.y)
    }
    fn z(&mut self, self_: Resource<QuatRes>) -> wasm_bridge::Result<f32> {
        let quat = self.table.get(&self_)?;
        Ok(quat.data.z)
    }
    fn w(&mut self, self_: Resource<QuatRes>) -> wasm_bridge::Result<f32> {
        let quat = self.table.get(&self_)?;
        Ok(quat.data.w)
    }

    fn set(
        &mut self,
        self_: Resource<QuatRes>,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    ) -> wasm_bridge::Result<()> {
        let quat = self.table.get_mut(&self_)?;
        quat.data.x = x;
        quat.data.y = y;
        quat.data.z = z;
        quat.data.w = w;

        quat.dirty.set(true);
        Ok(())
    }
    fn set_x(&mut self, self_: Resource<QuatRes>, x: f32) -> wasm_bridge::Result<()> {
        let quat = self.table.get_mut(&self_)?;
        quat.data.x = x;

        quat.dirty.set(true);
        Ok(())
    }
    fn set_y(&mut self, self_: Resource<QuatRes>, y: f32) -> wasm_bridge::Result<()> {
        let quat = self.table.get_mut(&self_)?;
        quat.data.y = y;

        quat.dirty.set(true);
        Ok(())
    }
    fn set_z(&mut self, self_: Resource<QuatRes>, z: f32) -> wasm_bridge::Result<()> {
        let quat = self.table.get_mut(&self_)?;
        quat.data.z = z;

        quat.dirty.set(true);
        Ok(())
    }
    fn set_w(&mut self, self_: Resource<QuatRes>, w: f32) -> wasm_bridge::Result<()> {
        let quat = self.table.get_mut(&self_)?;
        quat.data.w = w;

        quat.dirty.set(true);
        Ok(())
    }

    fn eq(
        &mut self,
        self_: Resource<QuatRes>,
        other: Resource<QuatRes>,
    ) -> wasm_bridge::Result<bool> {
        let quat_a = self.table.get(&self_)?;
        let quat_b = self.table.get(&other)?;
        Ok(quat_a.data == quat_b.data)
    }
    fn mul(
        &mut self,
        self_: Resource<QuatRes>,
        other: Resource<QuatRes>,
    ) -> wasm_bridge::Result<()> {
        let quat_a = self.table.get(&self_)?;
        let quat_b = self.table.get(&other)?;
        let result = quat_a.data.mul_quat(quat_b.data);

        let quat_a = self.table.get_mut(&self_)?;
        quat_a.data.clone_from(&result);

        quat_a.dirty.set(true);
        Ok(())
    }

    fn drop(&mut self, _rep: Resource<QuatRes>) -> wasm_bridge::Result<()> {
        // TODO: reference count resources, remove from table on drop if 0
        Ok(())
    }
}
