use std::collections::BTreeMap;

use loro_surgeon::{Hydrate, Reconcile};
use wired_records::HydratedHash;

use crate::state::vec2::HydratedVec2;

mod vec2;

#[derive(Hydrate, Reconcile, Default, Debug)]
pub struct SpaceState {
    portals: BTreeMap<String, PortalState>,
}

#[derive(Hydrate, Reconcile, Debug)]
pub struct PortalState {
    dest_portal: Option<HydratedHash>,
    dest_space: HydratedHash,
    size: HydratedVec2,
}
