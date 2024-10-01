use std::cell::Cell;

use bevy::{log::debug, prelude::*};
use wasm_bridge::component::{Resource, ResourceTable, ResourceTableError};

/// A `Cell<usize>` used for reference counting.
/// Defaults to a value of 1.
#[derive(Debug, Deref)]
pub struct RefCountCell(Cell<usize>);

impl Default for RefCountCell {
    fn default() -> Self {
        Self(Cell::new(1))
    }
}

pub trait RefCount {
    fn ref_count(&self) -> &Cell<usize>;

    /// Increment the ref count by 1.
    /// Returns the new value;
    fn increment(&self) -> usize {
        let count = self.ref_count();
        let new = count.get() + 1;
        count.set(new);
        new
    }

    /// Decrement the ref count by 1.
    /// Returns the new value;
    fn decrement(&self) -> usize {
        let count = self.ref_count();
        let val = count.get();
        debug_assert!(val > 0, "Cannot decrement, ref count already at 0");

        let new = val - 1;
        count.set(new);
        new
    }
}

/// A WASM resource that uses reference counting for its lifecycle.
/// This allows multiple copies of a resource to be created for the same data, while
/// ensuring the data is only dropped once all references are dropped.
///
/// New copies of the resource **must** be created using methods from this trait, to ensure
/// the reference count is accurate.
pub trait RefResource: RefCount + Send + Sized + 'static {
    fn new_own(&self, rep: u32) -> Resource<Self> {
        self.increment();
        Resource::new_own(rep)
    }

    fn from_res(
        res: &Resource<Self>,
        table: &ResourceTable,
    ) -> Result<Resource<Self>, ResourceTableError> {
        let data = table.get::<Self>(res)?;
        Ok(data.new_own(res.rep()))
    }

    fn from_rep(rep: u32, table: &ResourceTable) -> Result<Resource<Self>, ResourceTableError> {
        Self::from_res(&Resource::new_own(rep), table)
    }

    /// Decrement the reference count, dropping the resource if there are no more references.
    /// Returns a boolean indicating whether the resource was dropped.
    fn handle_drop(res: Resource<Self>, table: &mut ResourceTable) -> wasm_bridge::Result<bool> {
        let data = table.get(&res)?;
        let count = data.decrement();

        // Table owns a copy of the resource, so we delete when it is the only ref left.
        if count == 1 {
            debug!("Dropping res {}", res.rep());
            table.delete(res)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{data::ScriptData, load::DefaultMaterial};

    use super::*;

    pub fn init_test_data() -> (World, ScriptData) {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();

        let default_material = Handle::default();
        world.insert_resource(DefaultMaterial(default_material.clone()));

        let data = ScriptData::default();

        (world, data)
    }

    pub fn test_drop<T: RefResource + Send>(data: &mut ScriptData, res_a: Resource<T>) {
        let res_b = T::from_res(&res_a, &data.table).unwrap();

        let dummy = Resource::new_own(res_a.rep());
        assert!(data.table.get::<T>(&dummy).is_ok());

        T::handle_drop(res_a, &mut data.table).unwrap();
        assert!(data.table.get::<T>(&dummy).is_ok());

        T::handle_drop(res_b, &mut data.table).unwrap();
        let err = data.table.get::<T>(&dummy);
        assert!(err.is_err());
    }

    pub fn test_new<T: RefResource + Send>(data: &mut ScriptData, res_a: Resource<T>) {
        let a = data.table.get(&res_a).unwrap();
        let res_b = a.new_own(res_a.rep());
        assert_eq!(res_a.rep(), res_b.rep());

        let res_c = T::from_res(&res_a, &data.table).unwrap();
        assert_eq!(res_a.rep(), res_c.rep());

        let res_d = T::from_rep(res_a.rep(), &data.table).unwrap();
        assert_eq!(res_a.rep(), res_d.rep());
    }
}
