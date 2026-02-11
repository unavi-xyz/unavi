use bevy::{light::NotShadowCaster, prelude::*};

const CROSSHAIR_RADIUS: f32 = 0.005;

#[derive(Component)]
#[require(Visibility, Transform, CrosshairMode)]
pub struct Crosshair;

#[derive(Component, Default, PartialEq, Eq, Clone, Copy)]
pub enum CrosshairMode {
    Active,
    #[default]
    Inactive,
}

#[derive(Component)]
pub struct CrosshairMeshes {
    active: Handle<Mesh>,
    inactive: Handle<Mesh>,
}

pub(crate) fn spawn_crosshair(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mat = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        ..default()
    });

    let torus_radius = CROSSHAIR_RADIUS * 0.75;
    let torus = Torus::new(torus_radius, 2.0 * torus_radius)
        .mesh()
        .minor_resolution(8)
        .major_resolution(16);
    let active = meshes.add(torus);

    let sphere = Sphere::new(CROSSHAIR_RADIUS)
        .mesh()
        .ico(3)
        .expect("build ico sphere");
    let inactive = meshes.add(sphere);

    commands.spawn((
        Crosshair,
        Visibility::Hidden,
        NotShadowCaster,
        MeshMaterial3d(mat),
        Mesh3d(inactive.clone()),
        CrosshairMeshes { active, inactive },
    ));
}

pub(crate) fn set_crosshair_mesh(
    mut crosshair: Query<(&mut Mesh3d, &CrosshairMeshes, &CrosshairMode)>,
    mut prev: Local<CrosshairMode>,
) {
    let Ok((mut mesh, meshes, mode)) = crosshair.single_mut() else {
        return;
    };

    if *mode == *prev {
        return;
    }

    *prev = *mode;

    if *mode == CrosshairMode::Active {
        mesh.0 = meshes.active.clone();
    } else {
        mesh.0 = meshes.inactive.clone();
    }
}
