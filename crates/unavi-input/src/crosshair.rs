use bevy::prelude::*;

#[derive(Component)]
#[require(Visibility, Transform, FixedTargetTranslation)]
pub struct Crosshair;

const CROSSHAIR_RADIUS: f32 = 0.004;
const CROSSHAIR_WIDTH: f32 = 0.005;

pub fn spawn_crosshair(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mat = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        depth_bias: 10.0,
        unlit: true,
        ..default()
    });

    let torus = Torus::new(CROSSHAIR_RADIUS, CROSSHAIR_RADIUS + CROSSHAIR_WIDTH).mesh();
    let mesh = meshes.add(torus);

    commands.spawn((
        Crosshair,
        Visibility::Hidden,
        MeshMaterial3d(mat),
        Mesh3d(mesh),
    ));
}

/// Translation to be lerped to between each [`FixedUpdate`].
#[derive(Component, Default)]
pub struct FixedTargetTranslation(pub Vec3);

const LERP_SPEED: f32 = 30.0;

pub fn lerp_fixed_targets(
    time: Res<Time>,
    to_lerp: Query<(&mut Transform, &FixedTargetTranslation)>,
) {
    let t = (LERP_SPEED * time.delta_secs()).min(1.0);

    for (mut transform, target) in to_lerp {
        transform.translation = transform.translation.lerp(target.0, t);
    }
}
