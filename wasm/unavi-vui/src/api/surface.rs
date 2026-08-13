use crate::{
    api::{
        convert,
        drain,
        mote::Mote,
        put_up,
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
    wired::error::types::Error,
};

pub struct Orbit(SurfaceId);

impl GuestOrbit for Orbit {
    fn new(root: MoteBorrow<'_>, mount: Mount, capacity: u32) -> Result<OrbitHandle, Error> {
        let root = root.get::<Mote>().0.clone();
        let surface = put_up(|vui| vui.orbit(root, convert::mount(mount), capacity as usize))?;
        Ok(OrbitHandle::new(Self(surface)))
    }

    fn events(&self) -> Vec<Event> {
        drain(self.0)
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
        let root = root.get::<Mote>().0.clone();
        let surface =
            put_up(|vui| vui.grid(root, columns as usize, rows as usize, convert::mount(mount)))?;
        Ok(GridHandle::new(Self(surface)))
    }

    fn events(&self) -> Vec<Event> {
        drain(self.0)
    }
}
