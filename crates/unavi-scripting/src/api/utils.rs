use std::cell::Cell;

use bevy::log::warn;
use wasm_bridge::component::{Resource, ResourceTable, ResourceTableError};

pub trait RefCount {
    fn ref_count(&self) -> &Cell<usize>;

    fn increment(&self) -> usize {
        let count = self.ref_count();
        let new = count.get() + 1;
        count.set(new);
        new
    }
    fn decrement(&self) -> usize {
        let count = self.ref_count();
        let val = count.get();

        if val == 0 {
            warn!("Ref count already at 0");
            return 0;
        }

        let new = val - 1;
        count.set(new);
        new
    }
}

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

    fn handle_drop(res: Resource<Self>, table: &mut ResourceTable) -> wasm_bridge::Result<()> {
        let data = table.get(&res)?;
        let count = data.decrement();

        // Table owns a copy of the resource.
        if count == 1 {
            table.delete(res)?;
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::state::StoreState;

    use super::*;

    pub fn test_drop<T: RefResource + Send>(state: &mut StoreState, res_a: Resource<T>) {
        let res_b = T::from_res(&res_a, &state.table).unwrap();

        let dummy = Resource::new_own(res_a.rep());
        assert!(state.table.get::<T>(&dummy).is_ok());

        T::handle_drop(res_a, &mut state.table).unwrap();
        assert!(state.table.get::<T>(&dummy).is_ok());

        T::handle_drop(res_b, &mut state.table).unwrap();
        let err = state.table.get::<T>(&dummy);
        assert!(err.is_err());
    }
}
