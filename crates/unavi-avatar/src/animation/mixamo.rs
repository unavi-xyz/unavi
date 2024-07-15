use bevy::{animation::AnimationTargetId, prelude::*, utils::HashMap};
use bevy_vrm::BoneName;

pub struct MixamoAnimationTargets(pub HashMap<BoneName, AnimationTargetId>);

macro_rules! finger {
    ($map:ident, $chain:ident, $side:ident, $finger_vrm:ident, $finger_chain:ident) => {
        paste::paste! {
            {
                let mut chain = $chain.clone();

                $map.insert(
                    BoneName::[<$side $finger_vrm Proximal>],
                    chain.push_target(concat!(stringify!($side), "Hand", stringify!($finger_chain), "1")),
                );
                $map.insert(
                    BoneName::[<$side $finger_vrm Intermediate>],
                    chain.push_target(concat!(stringify!($side), "Hand", stringify!($finger_chain), "2")),
                );
                $map.insert(
                    BoneName::[<$side $finger_vrm Distal>],
                    chain.push_target(concat!(stringify!($side), "Hand", stringify!($finger_chain), "3")),
                );
            }
        }
    };
}

macro_rules! arm {
    ($map:ident, $chain:ident, $side:ident) => {
        paste::paste! {
            {
                let mut chain = $chain.clone();

                $map.insert(BoneName::[<$side Shoulder>], chain.push_target(concat!(stringify!($side), "Shoulder")));
                $map.insert(BoneName::[<$side UpperArm>], chain.push_target(concat!(stringify!($side), "Arm")));
                $map.insert(BoneName::[<$side LowerArm>], chain.push_target(concat!(stringify!($side), "ForeArm")));
                $map.insert(BoneName::[<$side Hand>], chain.push_target(concat!(stringify!($side), "Hand")));

                finger!($map, chain, $side, Thumb, Thumb);
                finger!($map, chain, $side, Index, Index);
                finger!($map, chain, $side, Middle, Middle);
                finger!($map, chain, $side, Ring, Ring);
                finger!($map, chain, $side, Little, Pinky);
            }
        }
    };
}

macro_rules! leg {
    ($map:ident, $chain:ident, $side:ident) => {
        paste::paste! {
            {
                let mut chain = $chain.clone();

                $map.insert(BoneName::[<$side UpperLeg>], chain.push_target(concat!(stringify!($side), "UpLeg")));
                $map.insert(BoneName::[<$side LowerLeg>], chain.push_target(concat!(stringify!($side), "Leg")));
                $map.insert(BoneName::[<$side Foot>], chain.push_target(concat!(stringify!($side), "Foot")));
                $map.insert(BoneName::[<$side Toes>], chain.push_target(concat!(stringify!($side), "ToeBase")));
            }
        }
    };
}

impl Default for MixamoAnimationTargets {
    fn default() -> Self {
        let mut map = HashMap::default();
        let mut chain = BoneChain::default();

        map.insert(BoneName::Hips, chain.push_target("Hips"));

        {
            let mut chain = chain.clone();

            map.insert(BoneName::Spine, chain.push_target("Spine"));
            map.insert(BoneName::Chest, chain.push_target("Spine1"));
            map.insert(BoneName::UpperChest, chain.push_target("Spine2"));

            {
                let mut chain = chain.clone();

                map.insert(BoneName::Neck, chain.push_target("Neck"));
                map.insert(BoneName::Head, chain.push_target("Head"));
            }

            arm!(map, chain, Left);
            arm!(map, chain, Right);
        }

        leg!(map, chain, Left);
        leg!(map, chain, Right);

        Self(map)
    }
}

#[derive(Default, Clone)]
struct BoneChain<'a>(Vec<&'a str>);

impl<'a> BoneChain<'a> {
    fn push_target(&mut self, name: &'a str) -> AnimationTargetId {
        self.0.push(name);
        self.target()
    }

    fn target(&self) -> AnimationTargetId {
        let mut names: Vec<Name> = vec!["Armature".into()];
        for value in self.0.iter() {
            names.push(format!("mixamorig:{}", value).into());
        }
        AnimationTargetId::from_names(names.iter())
    }
}

#[cfg(test)]
mod tests {
    use bevy::{gltf::GltfPlugin, render::mesh::skinning::SkinnedMeshInverseBindposes};

    use super::*;

    #[test]
    fn test_mixamo_targets() {
        let mut app = App::new();

        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: "../unavi-app/assets".to_string(),
                ..default()
            },
            AnimationPlugin,
            GltfPlugin::default(),
        ));

        app.init_asset::<Scene>();
        app.init_asset::<SkinnedMeshInverseBindposes>();

        app.add_systems(Startup, |asset_server: Res<AssetServer>| {
            let _: Handle<AnimationClip> =
                asset_server.load("models/character-animations.glb#Animation0");
        });

        app.add_systems(
            Update,
            |assets: Res<Assets<AnimationClip>>,
             time: Res<Time>,
             mut exit: EventWriter<AppExit>| {
                if time.elapsed_seconds() > 10.0 {
                    panic!("Took too long");
                }

                if let Some((_, clip)) = assets.iter().next() {
                    let targets = MixamoAnimationTargets::default();

                    for (name, target) in targets.0.into_iter() {
                        assert!(clip.curves_for_target(target).is_some(), "name={:?}", name);
                    }

                    exit.send_default();
                }
            },
        );

        app.run();
    }
}
