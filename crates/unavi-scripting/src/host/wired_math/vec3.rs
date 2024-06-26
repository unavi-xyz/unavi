use std::cell::Cell;

use bevy::math::Vec3;
use wasm_bridge::component::Resource;

use crate::state::StoreState;

use super::wired::math::types::HostVec3;

pub struct Vec3Res {
    pub data: Vec3,
    pub dirty: Cell<bool>,
    ref_count: usize,
}

impl Vec3Res {
    pub fn new(data: Vec3) -> Self {
        Self {
            data,
            dirty: Cell::new(true),
            ref_count: 1,
        }
    }
}

impl HostVec3 for StoreState {
    fn new(&mut self, x: f32, y: f32, z: f32) -> wasm_bridge::Result<Resource<Vec3Res>> {
        let res = self.table.push(Vec3Res::new(Vec3::new(x, y, z)))?;
        Ok(res)
    }
    fn default(&mut self) -> wasm_bridge::Result<Resource<Vec3Res>> {
        let res = self.table.push(Vec3Res::new(Vec3::default()))?;
        Ok(res)
    }
    fn splat(
        &mut self,
        value: f32,
    ) -> wasm_bridge::Result<wasm_bridge::component::Resource<Vec3Res>> {
        let res = self.table.push(Vec3Res::new(Vec3::splat(value)))?;
        Ok(res)
    }

    fn xyz(&mut self, self_: Resource<Vec3Res>) -> wasm_bridge::Result<(f32, f32, f32)> {
        let vec = self.table.get(&self_)?;
        Ok((vec.data.x, vec.data.y, vec.data.z))
    }
    fn x(&mut self, self_: Resource<Vec3Res>) -> wasm_bridge::Result<f32> {
        let vec = self.table.get(&self_)?;
        Ok(vec.data.x)
    }
    fn y(&mut self, self_: Resource<Vec3Res>) -> wasm_bridge::Result<f32> {
        let vec = self.table.get(&self_)?;
        Ok(vec.data.y)
    }
    fn z(&mut self, self_: Resource<Vec3Res>) -> wasm_bridge::Result<f32> {
        let vec = self.table.get(&self_)?;
        Ok(vec.data.z)
    }

    fn set(&mut self, self_: Resource<Vec3Res>, x: f32, y: f32, z: f32) -> wasm_bridge::Result<()> {
        let vec = self.table.get_mut(&self_)?;
        vec.data.x = x;
        vec.data.y = y;
        vec.data.z = z;

        vec.dirty.set(true);
        Ok(())
    }
    fn set_x(&mut self, self_: Resource<Vec3Res>, x: f32) -> wasm_bridge::Result<()> {
        let vec = self.table.get_mut(&self_)?;
        vec.data.x = x;

        vec.dirty.set(true);
        Ok(())
    }
    fn set_y(&mut self, self_: Resource<Vec3Res>, y: f32) -> wasm_bridge::Result<()> {
        let vec = self.table.get_mut(&self_)?;
        vec.data.y = y;
        Ok(())
    }
    fn set_z(&mut self, self_: Resource<Vec3Res>, z: f32) -> wasm_bridge::Result<()> {
        let vec = self.table.get_mut(&self_)?;
        vec.data.z = z;

        vec.dirty.set(true);
        Ok(())
    }

    fn eq(
        &mut self,
        self_: Resource<Vec3Res>,
        other: Resource<Vec3Res>,
    ) -> wasm_bridge::Result<bool> {
        let vec_a = self.table.get(&self_)?;
        let vec_b = self.table.get(&other)?;
        Ok(vec_a.data == vec_b.data)
    }

    fn drop(&mut self, _rep: Resource<Vec3Res>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
