use bevy::prelude::*;
use bevy_vrm::loader::Vrm;
use unavi_constants::player::{PLAYER_HEIGHT, PLAYER_WIDTH};

pub fn init_fallback_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let handle_mat = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        base_color: Color::linear_rgba(0.5, 0.8, 1.0, 0.4),
        perceptual_roughness: 1.0,
        unlit: true,
        ..Default::default()
    });
    commands.insert_resource(FallbackMaterial(handle_mat));

    let handle_mesh = meshes.add(Mesh::from(Capsule3d::new(
        PLAYER_WIDTH / 2.0,
        PLAYER_HEIGHT - PLAYER_WIDTH,
    )));
    commands.insert_resource(FallbackMesh(handle_mesh));
}

pub fn remove_fallback_avatar(
    mut commands: Commands,
    avatars: Query<Entity, (With<FallbackAvatar>, With<Handle<Vrm>>)>,
) {
    for entity in avatars.iter() {
        commands.entity(entity).remove::<FallbackAvatar>();
    }
}

pub fn spawn_fallback_children(
    fallback_material: Res<FallbackMaterial>,
    fallback_mesh: Res<FallbackMesh>,
    fallbacks: Query<Entity, Added<FallbackAvatar>>,
    mut commands: Commands,
) {
    for entity in fallbacks.iter() {
        commands.entity(entity).with_children(|commands| {
            commands.spawn((
                FallbackChild,
                PbrBundle {
                    material: fallback_material.clone(),
                    mesh: fallback_mesh.clone(),
                    transform: Transform::from_xyz(0.0, PLAYER_HEIGHT / 2.0, 0.0),
                    ..Default::default()
                },
            ));
        });
    }
}

pub fn despawn_fallback_children(
    children: Query<&Children>,
    fallback_children: Query<&FallbackChild>,
    mut commands: Commands,
    mut removed: RemovedComponents<FallbackAvatar>,
) {
    for entity in removed.read() {
        let children = match children.get(entity) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for child in children {
            if fallback_children.contains(*child) {
                commands.entity(*child).despawn();
            }
        }
    }
}

#[derive(Component, Default)]
pub struct FallbackAvatar;

#[derive(Resource, Deref)]
pub struct FallbackMaterial(Handle<StandardMaterial>);

#[derive(Resource, Deref)]
pub struct FallbackMesh(Handle<Mesh>);

#[derive(Component)]
pub struct FallbackChild;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_children() {
        let mut app = App::new();

        app.insert_resource(FallbackMaterial(Handle::default()))
            .insert_resource(FallbackMesh(Handle::default()))
            .add_systems(Update, (spawn_fallback_children, despawn_fallback_children));

        let entity = app.world_mut().spawn(FallbackAvatar).id();
        app.update();

        let children = app
            .world()
            .get::<Children>(entity)
            .expect("Children component not found");
        assert_eq!(children.len(), 1);

        let child = *children.iter().next().unwrap();
        assert!(app.world().get::<FallbackChild>(child).is_some());

        app.world_mut()
            .entity_mut(entity)
            .remove::<FallbackAvatar>();
        app.update();

        assert!(app.world().get_entity(child).is_none());
    }
}
