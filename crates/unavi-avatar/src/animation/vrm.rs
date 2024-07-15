use bevy::{animation::AnimationTargetId, utils::HashMap};
use bevy_vrm::BoneName;

use super::bone_chain::BoneChain;

pub struct VrmAnimationTargets(pub HashMap<BoneName, AnimationTargetId>);

macro_rules! push_bone {
    ($map:ident, $chain:ident, $name:ident) => {
        $map.insert(
            BoneName::$name,
            $chain.push_target(&serde_json::to_string(&BoneName::$name).unwrap()),
        );
    };
}

macro_rules! finger {
    ($map:ident, $chain:ident, $side:ident, $finger:ident) => {
        paste::paste! {
            {
                let mut chain = $chain.clone();

                push_bone!($map, chain, [<$side $finger Proximal>]);
                push_bone!($map, chain, [<$side $finger Intermediate>]);
                push_bone!($map, chain, [<$side $finger Distal>]);
            }
        }
    };
}

macro_rules! arm {
    ($map:ident, $chain:ident, $side:ident) => {
        paste::paste! {
            {
                let mut chain = $chain.clone();

                push_bone!($map, chain, [<$side Shoulder>]);
                push_bone!($map, chain, [<$side UpperArm>]);
                push_bone!($map, chain, [<$side LowerArm>]);
                push_bone!($map, chain, [<$side Hand>]);

                finger!($map, chain, $side, Thumb);
                finger!($map, chain, $side, Index);
                finger!($map, chain, $side, Middle);
                finger!($map, chain, $side, Ring);
                finger!($map, chain, $side, Little);
            }
        }
    };
}

macro_rules! leg {
    ($map:ident, $chain:ident, $side:ident) => {
        paste::paste! {
            {
                let mut chain = $chain.clone();

                push_bone!($map, chain, [<$side UpperLeg>]);
                push_bone!($map, chain, [<$side LowerLeg>]);
                push_bone!($map, chain, [<$side Foot>]);
                push_bone!($map, chain, [<$side Toes>]);
            }
        }
    };
}

impl Default for VrmAnimationTargets {
    fn default() -> Self {
        let mut map = HashMap::default();
        let mut chain = BoneChain::default();

        push_bone!(map, chain, Hips);

        leg!(map, chain, Left);
        leg!(map, chain, Right);

        push_bone!(map, chain, Spine);
        push_bone!(map, chain, Chest);
        push_bone!(map, chain, UpperChest);

        arm!(map, chain, Left);
        arm!(map, chain, Right);

        push_bone!(map, chain, Neck);
        push_bone!(map, chain, Head);

        {
            let mut chain = chain.clone();
            push_bone!(map, chain, Jaw);
        }

        {
            let mut chain = chain.clone();
            push_bone!(map, chain, LeftEye);
        }

        {
            let mut chain = chain.clone();
            push_bone!(map, chain, RightEye);
        }

        Self(map)
    }
}
