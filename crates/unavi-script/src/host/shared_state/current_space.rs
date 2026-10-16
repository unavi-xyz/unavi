//! The space the local user is in, for scripts to read.

use std::sync::Arc;

use bevy::prelude::*;
use parking_lot::RwLock;
use unavi_space::{
    grid::ActiveSpace,
    membership::{
        Space,
        SpaceId,
    },
};

/// The active space's id, or `None` while the user is between spaces.
#[derive(Resource, Clone, Default)]
pub struct CurrentSpace(Arc<RwLock<Option<SpaceId>>>);

impl CurrentSpace {
    #[must_use]
    pub fn get(&self) -> Option<SpaceId> {
        *self.0.read()
    }
}

/// Copies the active space's id out of the world, where a script can read it
/// without waiting on the world.
pub fn track_current_space(
    active: Option<Res<ActiveSpace>>,
    spaces: Query<&Space>,
    current: Res<CurrentSpace>,
) {
    let id = active
        .and_then(|active| active.0)
        .and_then(|entity| spaces.get(entity).ok())
        .map(Space::id);
    if current.get() != id {
        *current.0.write() = id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_the_active_space() {
        let mut app = App::new();
        app.init_resource::<CurrentSpace>()
            .init_resource::<ActiveSpace>()
            .add_systems(Update, track_current_space);
        let current = app.world().resource::<CurrentSpace>().clone();

        app.update();
        assert_eq!(current.get(), None, "no space is active yet");

        let id = SpaceId([7; 32]);
        let space = app.world_mut().spawn(Space(id)).id();
        app.world_mut().resource_mut::<ActiveSpace>().0 = Some(space);
        app.update();
        assert_eq!(current.get(), Some(id));

        app.world_mut().resource_mut::<ActiveSpace>().0 = None;
        app.update();
        assert_eq!(current.get(), None);
    }
}
