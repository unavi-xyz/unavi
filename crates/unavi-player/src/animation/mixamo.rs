use std::{cell::RefCell, rc::Rc, sync::LazyLock};

use bevy::{animation::AnimationTargetId, platform::collections::HashMap};
use bevy_vrm::{BoneName, animations::target_chain::TargetChain};

macro_rules! finger {
    ($chain:ident, $side:ident, $finger_vrm:ident, $finger_chain:ident) => {
        paste::paste! {
            {
                let mut chain = $chain.clone();

                chain.push_bone(
                    BoneName::[<$side $finger_vrm Proximal>],
                    concat!("mixamorig:", stringify!($side), "Hand", stringify!($finger_chain), "1"),
                );
                chain.push_bone(
                    BoneName::[<$side $finger_vrm Intermediate>],
                    concat!("mixamorig:", stringify!($side), "Hand", stringify!($finger_chain), "2"),
                );
                chain.push_bone(
                    BoneName::[<$side $finger_vrm Distal>],
                    concat!("mixamorig:", stringify!($side), "Hand", stringify!($finger_chain), "3"),
                );
            }
        }
    };
}

macro_rules! arm {
    ($chain:ident, $side:ident) => {
        paste::paste! {
            {
                let mut chain = $chain.clone();

                chain.push_bone(BoneName::[<$side Shoulder>], concat!("mixamorig:", stringify!($side), "Shoulder"));
                chain.push_bone(BoneName::[<$side UpperArm>], concat!("mixamorig:", stringify!($side), "Arm"));
                chain.push_bone(BoneName::[<$side LowerArm>], concat!("mixamorig:", stringify!($side), "ForeArm"));
                chain.push_bone(BoneName::[<$side Hand>], concat!("mixamorig:", stringify!($side), "Hand"));

                finger!(chain, $side, Thumb, Thumb);
                finger!(chain, $side, Index, Index);
                finger!(chain, $side, Middle, Middle);
                finger!(chain, $side, Ring, Ring);
                finger!(chain, $side, Little, Pinky);
            }
        }
    };
}

macro_rules! leg {
    ($chain:ident, $side:ident) => {
        paste::paste! {
            {
                let mut chain = $chain.clone();

                chain.push_bone(BoneName::[<$side UpperLeg>], concat!("mixamorig:", stringify!($side), "UpLeg"));
                chain.push_bone(BoneName::[<$side LowerLeg>], concat!("mixamorig:", stringify!($side), "Leg"));
                chain.push_bone(BoneName::[<$side Foot>], concat!("mixamorig:", stringify!($side), "Foot"));
                chain.push_bone(BoneName::[<$side Toes>], concat!("mixamorig:", stringify!($side), "ToeBase"));
            }
        }
    };
}

/// Wrapper around [TargetChain].
/// Allows us to re-use code for both [MIXAMO_ANIMATION_TARGETS] and [MIXAMO_BONE_NAMES].
#[derive(Clone)]
struct ChainWrapper<'a> {
    chain: TargetChain,
    names: Rc<RefCell<HashMap<BoneName, &'a str>>>,
    targets: Rc<RefCell<HashMap<BoneName, AnimationTargetId>>>,
}

impl<'a> ChainWrapper<'a> {
    fn new(chain: TargetChain) -> Self {
        Self {
            chain,
            names: Default::default(),
            targets: Default::default(),
        }
    }

    fn push_bone(&mut self, bone: BoneName, name: &'a str) {
        self.names.borrow_mut().insert(bone, name);
        let target = self.chain.push_target(name.to_string());
        self.targets.borrow_mut().insert(bone, target);
    }

    fn into_maps(
        self,
    ) -> (
        HashMap<BoneName, &'a str>,
        HashMap<BoneName, AnimationTargetId>,
    ) {
        (self.names.take(), self.targets.take())
    }
}

fn create_chain() -> ChainWrapper<'static> {
    let mut chain = TargetChain::default();
    chain.push_target("Armature".to_string());

    let mut chain = ChainWrapper::new(chain);

    chain.push_bone(BoneName::Hips, "mixamorig:Hips");

    leg!(chain, Left);
    leg!(chain, Right);

    chain.push_bone(BoneName::Spine, "mixamorig:Spine");
    chain.push_bone(BoneName::Chest, "mixamorig:Spine1");
    chain.push_bone(BoneName::UpperChest, "mixamorig:Spine2");

    arm!(chain, Left);
    arm!(chain, Right);

    chain.push_bone(BoneName::Neck, "mixamorig:Neck");
    chain.push_bone(BoneName::Head, "mixamorig:Head");

    chain
}

pub static MIXAMO_ANIMATION_TARGETS: LazyLock<HashMap<BoneName, AnimationTargetId>> =
    LazyLock::new(|| {
        let chain = create_chain();
        let (_, targets) = chain.into_maps();
        targets
    });

pub static MIXAMO_BONE_NAMES: LazyLock<HashMap<BoneName, &'static str>> = LazyLock::new(|| {
    let chain = create_chain();
    let (names, _) = chain.into_maps();
    names
});

#[cfg(test)]
mod tests {
    // TODO: This test sometimes fails from a panic while running the main bevy schedule
    // use bevy::{gltf::GltfPlugin, prelude::*, render::mesh::skinning::SkinnedMeshInverseBindposes};
    //
    // use super::*;
    //
    // #[test]
    // fn test_mixamo_targets() {
    //     let mut app = App::new();
    //
    //     app.add_plugins((
    //         MinimalPlugins,
    //         AssetPlugin {
    //             file_path: "../unavi/assets".to_string(),
    //             ..default()
    //         },
    //         AnimationPlugin,
    //         GltfPlugin::default(),
    //     ));
    //
    //     app.init_asset::<Scene>();
    //     app.init_asset::<SkinnedMeshInverseBindposes>();
    //
    //     let asset_server = app.world().get_resource::<AssetServer>().unwrap();
    //     let _ = asset_server.load::<AnimationClip>("character-animations.glb#Animation0");
    //
    //     app.add_systems(
    //         Update,
    //         |assets: Res<Assets<AnimationClip>>,
    //          time: Res<Time>,
    //          mut exit: EventWriter<AppExit>| {
    //             if time.elapsed_seconds() > 15.0 {
    //                 panic!("Took too long");
    //             }
    //
    //             if let Some((_, clip)) = assets.iter().next() {
    //                 for (name, target) in MIXAMO_ANIMATION_TARGETS.iter() {
    //                     assert!(clip.curves_for_target(*target).is_some(), "name={:?}", name);
    //                 }
    //
    //                 exit.send_default();
    //             }
    //         },
    //     );
    //
    //     app.run();
    // }
}
