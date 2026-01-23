use bevy::{
    pbr::{Atmosphere, AtmosphereSettings},
    prelude::*,
};
use bevy_hsd::{
    Stage,
    attributes::xform::Xform,
    stage::{Attrs, LayerData, NodeData, OpinionData, StageData},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use loro::TreeID;
use wired_schemas::conv::tree::TreeNode;

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
    let stage = StageData {
        layers: vec![LayerData {
            nodes: vec![TreeNode {
                id: TreeID {
                    peer: 1,
                    counter: 2,
                },
                parent: None,
                meta: NodeData { id: "a".into() },
            }],
            opinions: vec![OpinionData {
                id: "a".into(),
                attrs: Attrs {
                    xform: Some(Xform {
                        rotate: None,
                        scale: None,
                        translate: None,
                    }),
                    ..Default::default()
                },
            }],
        }],
    };

    commands.spawn(Stage(stage));
}
