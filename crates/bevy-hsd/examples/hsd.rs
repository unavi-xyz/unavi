use bevy::{
    pbr::{Atmosphere, AtmosphereSettings},
    prelude::*,
};
use bevy_hsd::{
    Stage,
    stage::{LayerData, OpinionData, StageData},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use loro::{LoroListValue, LoroMapValue, LoroValue};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))
        .add_systems(Startup, (setup_scene, load_hsd))
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.0, 5.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
        Atmosphere::EARTH,
        AtmosphereSettings::default(),
    ));
}

fn load_hsd(mut commands: Commands) {
    let mut attrs_0 = LoroMapValue::default();

    let mut xform_pos = LoroListValue::default();
    *xform_pos.make_mut() = vec![
        LoroValue::Double(1.0),
        LoroValue::Double(2.0),
        LoroValue::Double(3.0),
    ];

    let mut attrs_0_xform = LoroMapValue::default();
    attrs_0_xform
        .make_mut()
        .insert("position".to_string(), LoroValue::List(xform_pos));

    attrs_0
        .make_mut()
        .insert("xform".to_string(), LoroValue::Map(attrs_0_xform));

    let stage = StageData {
        layers: vec![LayerData {
            enabled: true,
            opinions: vec![OpinionData {
                node: 0,
                attrs: attrs_0,
            }],
        }],
    };

    commands.spawn(Stage(stage));
}
