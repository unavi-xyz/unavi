use bevy::prelude::*;

use crate::FallbackAvatar;

const FALLBACK_RADIUS: f32 = 0.4;
const FALLBACK_HEIGHT: f32 = 1.8;
const CAPSULE_LENGTH: f32 = FALLBACK_HEIGHT - (FALLBACK_RADIUS * 2.0);

#[derive(Resource, Deref)]
pub struct FallbackMaterial(Handle<StandardMaterial>);

pub fn init_fallback_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let handle = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        base_color: Color::rgba(0.5, 0.8, 1.0, 0.5),
        perceptual_roughness: 1.0,
        unlit: true,
        ..Default::default()
    });

    commands.insert_resource(FallbackMaterial(handle));
}

pub fn spawn_fallbacks(
    fallback_material: Res<FallbackMaterial>,
    fallbacks: Query<Entity, Added<FallbackAvatar>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for entity in fallbacks.iter() {
        commands.entity(entity).insert(PbrBundle {
            material: fallback_material.clone(),
            mesh: meshes.add(Mesh::from(Capsule3d::new(FALLBACK_RADIUS, CAPSULE_LENGTH))),
            transform: Transform::from_xyz(0.0, FALLBACK_HEIGHT / 2.0, 0.0),
            ..Default::default()
        });
    }
}
