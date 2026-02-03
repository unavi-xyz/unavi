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

/// Wrapper around [`TargetChain`].
/// Allows code re-use for both animation targets and bone names.
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
            names: Rc::default(),
            targets: Rc::default(),
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

pub static MIXAMO_BONE_NAMES: LazyLock<HashMap<BoneName, &'static str>> = LazyLock::new(|| {
    let chain = create_chain();
    let (names, _) = chain.into_maps();
    names
});
