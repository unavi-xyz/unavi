use bevy::prelude::*;

use crate::FallbackAvatar;

const FALLBACK_RADIUS: f32 = 0.4;
const FALLBACK_HEIGHT: f32 = 1.8;
const CAPSULE_LENGTH: f32 = FALLBACK_HEIGHT - (FALLBACK_RADIUS * 2.0);

#[derive(Resource, Deref)]
pub struct FallbackMaterial(Handle<StandardMaterial>);

#[derive(Resource, Deref)]
pub struct FallbackMesh(Handle<Mesh>);

pub fn init_fallback_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let handle_mat = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        base_color: Color::rgba(0.5, 0.8, 1.0, 0.5),
        perceptual_roughness: 1.0,
        unlit: true,
        ..Default::default()
    });
    commands.insert_resource(FallbackMaterial(handle_mat));

    let handle_mesh = meshes.add(Mesh::from(Capsule3d::new(FALLBACK_RADIUS, CAPSULE_LENGTH)));
    commands.insert_resource(FallbackMesh(handle_mesh));
}

#[derive(Component, Deref)]
pub struct FallbackChild(Entity);

pub fn spawn_fallbacks(
    fallback_material: Res<FallbackMaterial>,
    fallback_mesh: Res<FallbackMesh>,
    fallbacks: Query<Entity, Added<FallbackAvatar>>,
    mut commands: Commands,
) {
    for entity in fallbacks.iter() {
        let child = commands
            .spawn(PbrBundle {
                material: fallback_material.clone(),
                mesh: fallback_mesh.clone(),
                transform: Transform::from_xyz(0.0, FALLBACK_HEIGHT / 2.0, 0.0),
                ..Default::default()
            })
            .id();

        commands.entity(entity).insert(FallbackChild(child));
    }
}
pub fn despawn_fallbacks(
    mut commands: Commands,
    removed: Query<(Entity, &FallbackChild), Without<FallbackAvatar>>,
) {
    for (entity, child) in removed.iter() {
        commands.entity(**child).despawn();
        commands.entity(entity).remove::<FallbackChild>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_despawn() {
        let mut app = App::new();

        app.insert_resource(FallbackMaterial(Handle::default()))
            .insert_resource(FallbackMesh(Handle::default()))
            .add_systems(Update, (spawn_fallbacks, despawn_fallbacks));

        let entity = app.world.spawn(FallbackAvatar).id();
        app.update();

        let child = app
            .world
            .get::<FallbackChild>(entity)
            .expect("FallbackChild component not found")
            .0;

        app.world.entity_mut(entity).remove::<FallbackAvatar>();
        app.update();

        assert!(app.world.get::<FallbackChild>(entity).is_none());
        assert!(app.world.get_entity(child).is_none());
    }
}
