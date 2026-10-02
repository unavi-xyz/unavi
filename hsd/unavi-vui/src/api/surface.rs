use crate::{
    api::{
        convert,
        dismiss,
        drain,
        mote::Mote,
        put_up,
        remove,
        shown,
        summon,
    },
    exports::unavi::vui::api::{
        Event,
        Grid as GridHandle,
        GuestGrid,
        GuestOrbit,
        MoteBorrow,
        Mount,
        Orbit as OrbitHandle,
    },
    scene::SurfaceId,
    wired::core::error::Error,
};

/// `orbit.new`: `capacity` is at most 64.
const MAX_ORBIT_CAPACITY: u32 = 64;
/// `grid.new`: `columns * rows` is at most 256.
const MAX_GRID_CELLS: u32 = 256;

pub struct Orbit(SurfaceId);

impl GuestOrbit for Orbit {
    fn new(root: MoteBorrow<'_>, mount: Mount, capacity: u32) -> Result<OrbitHandle, Error> {
        if capacity > MAX_ORBIT_CAPACITY {
            return Err(Error::InvalidArgument(format!(
                "capacity is {capacity}, over the limit of {MAX_ORBIT_CAPACITY}"
            )));
        }
        let root = root.get::<Mote>().0.clone();
        let surface = put_up(|vui| vui.orbit(root, convert::mount(mount), capacity as usize))?;
        Ok(OrbitHandle::new(Self(surface)))
    }

    fn events(&self) -> Vec<Event> {
        drain(self.0)
    }

    fn summon(&self) -> Result<(), Error> {
        summon(self.0)
    }

    fn dismiss(&self) -> Result<(), Error> {
        dismiss(self.0)
    }

    fn shown(&self) -> bool {
        shown(self.0)
    }
}

/// Frees the surface's prims once nothing holds this handle, rather than
/// leaving it drawn and stepped forever.
impl Drop for Orbit {
    fn drop(&mut self) {
        remove(self.0);
    }
}

pub struct Grid(SurfaceId);

impl GuestGrid for Grid {
    fn new(
        root: MoteBorrow<'_>,
        columns: u32,
        rows: u32,
        mount: Mount,
    ) -> Result<GridHandle, Error> {
        if columns.saturating_mul(rows) > MAX_GRID_CELLS {
            return Err(Error::InvalidArgument(format!(
                "a {columns}x{rows} grid is {} cells, over the limit of {MAX_GRID_CELLS}",
                columns.saturating_mul(rows)
            )));
        }
        let root = root.get::<Mote>().0.clone();
        let surface =
            put_up(|vui| vui.grid(root, columns as usize, rows as usize, convert::mount(mount)))?;
        Ok(GridHandle::new(Self(surface)))
    }

    fn events(&self) -> Vec<Event> {
        drain(self.0)
    }

    fn summon(&self) -> Result<(), Error> {
        summon(self.0)
    }

    fn dismiss(&self) -> Result<(), Error> {
        dismiss(self.0)
    }

    fn shown(&self) -> bool {
        shown(self.0)
    }
}

/// Frees the surface's prims once nothing holds this handle, rather than
/// leaving it drawn and stepped forever.
impl Drop for Grid {
    fn drop(&mut self) {
        remove(self.0);
    }
}
