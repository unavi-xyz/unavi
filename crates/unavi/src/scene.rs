use bevy::{prelude::*, render::mesh::VertexAttributeValues};

pub fn spawn_lights(mut commands: Commands, mut ambient: ResMut<AmbientLight>) {
    ambient.brightness = 40.0;
    ambient.color = Color::linear_rgb(0.95, 0.95, 1.0);

    commands.spawn((
        DirectionalLight {
            color: Color::linear_rgb(1.0, 1.0, 0.98),
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.5, 10.0, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn spawn_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let ground_size = 30.0;

    let ground_texture = asset_server.load("images/dev-white.png");
    let ground_texture_scale = ground_size / 4.0;

    let mut ground_mesh = Mesh::from(Cuboid {
        half_size: Vec3::new(ground_size / 2.0, 0.05, ground_size / 2.0),
    });

    match ground_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0).unwrap() {
        VertexAttributeValues::Float32x2(uvs) => {
            for uv in uvs {
                uv[0] *= ground_texture_scale;
                uv[1] *= ground_texture_scale;
            }
        }
        _ => unreachable!(),
    }

    commands.spawn((
        Mesh3d(meshes.add(ground_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(ground_texture),
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    commands.spawn((Camera3d::default(), Transform::from_xyz(0.0, 1.0, 0.0)));
}
