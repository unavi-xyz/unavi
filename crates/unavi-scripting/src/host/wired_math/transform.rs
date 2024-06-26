use bevy::math::{Quat, Vec3};
use wasm_bridge::component::{Resource, ResourceTable, ResourceTableError};

use crate::state::StoreState;

use super::{quat::QuatRes, vec3::Vec3Res, wired::math::types::HostTransform};

pub struct Transform {
    pub rotation: Resource<QuatRes>,
    pub scale: Resource<Vec3Res>,
    pub translation: Resource<Vec3Res>,
}

impl Transform {
    /// Cleans all resources.
    /// Returns true if any were dirty.
    pub fn clean(&self, table: &ResourceTable) -> Result<bool, ResourceTableError> {
        let mut dirty = false;

        let translation = table.get(&self.translation)?;
        if translation.dirty.get() {
            dirty = true;
            translation.dirty.set(false);
        }

        let rotation = table.get(&self.rotation)?;
        if rotation.dirty.get() {
            dirty = true;
            rotation.dirty.set(false);
        }

        let scale = table.get(&self.scale)?;
        if scale.dirty.get() {
            dirty = true;
            scale.dirty.set(false);
        }

        Ok(dirty)
    }
}

// TODO: Could we use unsafe rust for the Vec3s and Quat, to read and write directly to the
// Bevy ECS? As long as script execution is run as an exclusive system.

impl HostTransform for StoreState {
    fn new(
        &mut self,
        translation: Resource<Vec3Res>,
        rotation: Resource<QuatRes>,
        scale: Resource<Vec3Res>,
    ) -> wasm_bridge::Result<Resource<Transform>> {
        let res = self.table.push(Transform {
            translation,
            rotation,
            scale,
        })?;
        Ok(res)
    }
    fn default(&mut self) -> wasm_bridge::Result<Resource<Transform>> {
        let rotation = self.table.push(QuatRes::new(Quat::default()))?;
        let scale = self.table.push(Vec3Res::new(Vec3::splat(1.0)))?;
        let translation = self.table.push(Vec3Res::new(Vec3::default()))?;
        let res = self.table.push(Transform {
            rotation,
            scale,
            translation,
        })?;
        Ok(res)
    }

    fn rotation(&mut self, self_: Resource<Transform>) -> wasm_bridge::Result<Resource<QuatRes>> {
        let res = self.table.get(&self_)?;
        let rotation = self.table.get(&res.rotation)?;
        Ok(rotation.new_own(res.rotation.rep()))
    }
    fn scale(&mut self, self_: Resource<Transform>) -> wasm_bridge::Result<Resource<Vec3Res>> {
        let res = self.table.get(&self_)?;
        Ok(Resource::new_own(res.scale.rep()))
    }
    fn translation(
        &mut self,
        self_: Resource<Transform>,
    ) -> wasm_bridge::Result<Resource<Vec3Res>> {
        let res = self.table.get(&self_)?;
        Ok(Resource::new_own(res.translation.rep()))
    }

    fn eq(
        &mut self,
        self_: Resource<Transform>,
        other: Resource<Transform>,
    ) -> wasm_bridge::Result<bool> {
        let tr_a = self.table.get(&self_)?;
        let rot_a = self.table.get(&tr_a.rotation)?;
        let sca_a = self.table.get(&tr_a.scale)?;
        let tra_a = self.table.get(&tr_a.translation)?;

        let tr_b = self.table.get(&other)?;
        let rot_b = self.table.get(&tr_b.rotation)?;
        let sca_b = self.table.get(&tr_b.scale)?;
        let tra_b = self.table.get(&tr_b.translation)?;

        Ok((rot_a.data == rot_b.data) && (sca_a.data == sca_b.data) && (tra_a.data == tra_b.data))
    }

    fn drop(&mut self, _rep: Resource<Transform>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean() {
        let mut table = ResourceTable::default();

        let rotation = table.push(QuatRes::new(Quat::default())).unwrap();
        let scale = table.push(Vec3Res::new(Vec3::default())).unwrap();
        let translation = table.push(Vec3Res::new(Vec3::default())).unwrap();

        let transform = Transform {
            rotation,
            scale,
            translation,
        };

        let rotation_quat = table.get_mut(&transform.rotation).unwrap();
        rotation_quat.dirty.set(true);
        assert!(transform.clean(&table).unwrap());
        let rotation_quat = table.get(&transform.rotation).unwrap();
        assert!(!rotation_quat.dirty.get());
        assert!(!transform.clean(&table).unwrap());

        let scale_vec = table.get_mut(&transform.scale).unwrap();
        scale_vec.dirty.set(true);
        assert!(transform.clean(&table).unwrap());
        let scale_vec = table.get(&transform.scale).unwrap();
        assert!(!scale_vec.dirty.get());
        assert!(!transform.clean(&table).unwrap());

        let translation_vec = table.get_mut(&transform.translation).unwrap();
        translation_vec.dirty.set(true);
        assert!(transform.clean(&table).unwrap());
        let transform_vec = table.get(&transform.scale).unwrap();
        assert!(!transform_vec.dirty.get());
        assert!(!transform.clean(&table).unwrap());

        let rotation_quat = table.get_mut(&transform.rotation).unwrap();
        rotation_quat.dirty.set(true);
        let scale_vec = table.get_mut(&transform.scale).unwrap();
        scale_vec.dirty.set(true);
        let translation_vec = table.get_mut(&transform.translation).unwrap();
        translation_vec.dirty.set(true);
        assert!(transform.clean(&table).unwrap());
        let rotation_quat = table.get(&transform.rotation).unwrap();
        assert!(!rotation_quat.dirty.get());
        let scale_vec = table.get(&transform.scale).unwrap();
        assert!(!scale_vec.dirty.get());
        let transform_vec = table.get(&transform.scale).unwrap();
        assert!(!transform_vec.dirty.get());
        assert!(!transform.clean(&table).unwrap());
    }
}
