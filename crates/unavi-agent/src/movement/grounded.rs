use avian3d::prelude::{
    Collider,
    RigidBody,
};
use bevy::prelude::*;
use bevy_tnua::{
    TnuaNotPlatform,
    prelude::TnuaController,
};

use crate::{
    AgentRig,
    ControlScheme,
    Grounded,
};

/// A collider with no rigid body: query-only geometry, such as a UI hit
/// surface.
#[derive(Component)]
pub struct QueryOnlyCollider;

/// Keeps query-only colliders out of Tnua's ground detection.
///
/// Tnua's shape cast treats any collider it meets as a platform, and a halo
/// summoned below the eye is a floor only it can see: the rig is lifted to
/// float height above the menu and, because the menu travels with the body,
/// keeps rising. A collider that gains a body is a platform again.
pub fn exclude_query_only_colliders(
    mut commands: Commands,
    bodyless: Query<
        Entity,
        (
            With<Collider>,
            Without<RigidBody>,
            Without<QueryOnlyCollider>,
        ),
    >,
    bodied: Query<Entity, (With<Collider>, With<RigidBody>, With<QueryOnlyCollider>)>,
) {
    for entity in &bodyless {
        commands
            .entity(entity)
            .insert((TnuaNotPlatform, QueryOnlyCollider));
    }
    for entity in &bodied {
        commands
            .entity(entity)
            .remove::<(TnuaNotPlatform, QueryOnlyCollider)>();
    }
}

pub fn sync_grounded_state(
    mut rigs: Query<(&TnuaController<ControlScheme>, &mut Grounded), With<AgentRig>>,
) {
    for (controller, mut grounded) in &mut rigs {
        if let Ok(airborne) = controller.is_airborne() {
            grounded.0 = !airborne;
        }
    }
}

#[cfg(test)]
mod tests {
    use avian3d::prelude::{
        Collider,
        RigidBody,
    };
    use bevy::prelude::*;
    use bevy_tnua::TnuaNotPlatform;

    use super::{
        QueryOnlyCollider,
        exclude_query_only_colliders,
    };

    fn app() -> App {
        let mut app = App::new();
        app.add_systems(Update, exclude_query_only_colliders);
        app
    }

    #[test]
    fn a_bodyless_collider_is_not_a_platform() {
        let mut app = app();
        let collider = app.world_mut().spawn(Collider::sphere(0.1)).id();

        app.update();

        let entity = app.world().entity(collider);
        assert!(entity.contains::<TnuaNotPlatform>());
        assert!(entity.contains::<QueryOnlyCollider>());
    }

    #[test]
    fn a_collider_with_a_body_is_a_platform() {
        let mut app = app();
        let collider = app
            .world_mut()
            .spawn((Collider::sphere(0.1), RigidBody::Dynamic))
            .id();

        app.update();

        assert!(!app.world().entity(collider).contains::<TnuaNotPlatform>());
    }

    #[test]
    fn a_bodyless_collider_that_gains_a_body_becomes_a_platform() {
        let mut app = app();
        let collider = app.world_mut().spawn(Collider::sphere(0.1)).id();
        app.update();
        assert!(app.world().entity(collider).contains::<TnuaNotPlatform>());

        app.world_mut()
            .entity_mut(collider)
            .insert(RigidBody::Dynamic);
        app.update();

        let entity = app.world().entity(collider);
        assert!(!entity.contains::<TnuaNotPlatform>());
        assert!(!entity.contains::<QueryOnlyCollider>());
    }
}
