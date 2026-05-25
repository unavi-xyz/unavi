use bevy::{
    light::{
        NotShadowCaster,
        NotShadowReceiver,
    },
    prelude::*,
    render::render_resource::{
        AsBindGroup,
        ShaderType,
    },
    shader::ShaderRef,
};

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct SkyMaterial {
    #[uniform(0)]
    pub params: SkyParams,
}

#[derive(Clone, Copy, ShaderType, Debug)]
pub struct SkyParams {
    pub top_color:        Vec4,
    pub bottom_color:     Vec4,
    pub horizon_softness: f32,
    pub radial_falloff:   f32,
}

impl Material for SkyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader/sky.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

const SKY_RADIUS: f32 = 1.0e5;

pub fn spawn_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    let mesh = meshes.add(Sphere::new(SKY_RADIUS).mesh().ico(5).expect("sky sphere"));

    let material = materials.add(SkyMaterial {
        params: SkyParams {
            top_color:        Vec4::new(0.75, 0.78, 0.82, 1.0),
            bottom_color:     Vec4::new(0.38, 0.40, 0.42, 1.0),
            horizon_softness: 0.05,
            radial_falloff:   0.1,
        },
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        NotShadowCaster,
        NotShadowReceiver,
        Transform::from_scale(Vec3::splat(-1.0)),
    ));
}
