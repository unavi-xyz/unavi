use bevy::platform::collections::HashMap;
use blake3::Hash;

pub struct SpaceState {
    portals: HashMap<String, PortalDestination>,
}

pub struct PortalDestination {
    space: Hash,
    portal: Option<Hash>,
}
