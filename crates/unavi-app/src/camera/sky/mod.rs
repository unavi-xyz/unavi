//! The sky: an overcast dome tinted by the space's sky colour, with a
//! graticule drawn over it in limbo.

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

pub mod blend;
pub mod palette;

use palette::Palette;

/// Noise cells per second the cloud deck drifts, along x and z.
const CLOUD_DRIFT: [f64; 2] = [0.012, 0.004];
/// Cells after which the shader's cloud noise repeats, so drift wraps there
/// without a seam.
const CLOUD_PERIOD: f64 = 256.0;
/// Graticule lines per second the dome turns.
const GRATICULE_SPIN: f64 = 0.12;
/// Seconds for the slowest graticule pulse to climb once. Faster pulses run
/// at whole multiples, so every phase wraps together.
const PULSE_PERIOD: f64 = 40.0;
const SKY_RADIUS: f32 = 1.0e5;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct SkyMaterial {
    #[uniform(0)]
    pub params: SkyParams,
}

#[derive(Clone, Copy, ShaderType, Debug)]
pub struct SkyParams {
    pub zenith:    Vec4,
    pub horizon:   Vec4,
    pub haze:      Vec4,
    pub line:      Vec4,
    pub glow:      Vec4,
    /// The cloud deck's offset, in noise cells.
    pub drift:     Vec2,
    pub graticule: f32,
    /// How far the graticule has turned, in lines, from 0 to 1.
    pub spin:      f32,
    /// The slowest pulse's progress up the dome, from 0 to 1.
    pub pulse:     f32,
}

impl SkyParams {
    // Phases are wrapped here in f64, so the shader never sees a clock that
    // jumps.
    fn new(palette: &Palette, graticule: f32, elapsed: f64) -> Self {
        let [drift_x, drift_z] =
            CLOUD_DRIFT.map(|rate| (elapsed * rate).rem_euclid(CLOUD_PERIOD) as f32);
        Self {
            zenith: palette.zenith.to_vec4(),
            horizon: palette.horizon.to_vec4(),
            haze: palette.haze.to_vec4(),
            line: palette.line.to_vec4(),
            glow: palette.glow.to_vec4(),
            drift: Vec2::new(drift_x, drift_z),
            graticule,
            spin: (elapsed * GRATICULE_SPIN).fract() as f32,
            pulse: (elapsed / PULSE_PERIOD).fract() as f32,
        }
    }
}

impl Material for SkyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader/sky.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

pub fn spawn_sky(
    blend: Res<blend::SkyBlend>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SkyMaterial>>,
) {
    let mesh = meshes.add(Sphere::new(SKY_RADIUS).mesh().uv(64, 32));

    let material = materials.add(SkyMaterial {
        params: SkyParams::new(&blend.palette(), blend.graticule(), 0.0),
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        NotShadowCaster,
        NotShadowReceiver,
        Transform::from_scale(Vec3::splat(-1.0)),
    ));
}
