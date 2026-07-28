use bevy::{
    asset::{
        AssetLoader,
        LoadContext,
        io::Reader,
    },
    platform::collections::HashMap,
    prelude::*,
};
use gltf::animation::util::ReadOutputs;
use thiserror::Error;

/// Raw rotation animations parsed directly from a glTF, used to retarget
/// Mixamo animations onto VRM humanoid bones.
#[derive(Asset, TypePath, Default)]
pub struct RawAnimations {
    pub animations: Vec<RawAnimation>,
    pub nodes:      HashMap<String, RawNode>,
}

pub struct RawAnimation {
    pub channels: Vec<RawRotationChannel>,
}

pub struct RawRotationChannel {
    pub target:     String,
    pub timestamps: Vec<f32>,
    pub values:     Vec<Quat>,
}

pub struct RawNode {
    pub rotation: Quat,
    pub parent:   Option<String>,
}

impl RawAnimations {
    /// Accumulated rest rotation of the ancestors of `node`, from root down to
    /// the direct parent.
    #[must_use]
    pub fn parent_rest(&self, node: &str) -> Quat {
        let mut chain = Vec::new();
        let mut cur = self.nodes.get(node).and_then(|n| n.parent.clone());
        while let Some(name) = &cur {
            let Some(parent) = self.nodes.get(name) else {
                break;
            };
            chain.push(parent.rotation);
            cur.clone_from(&parent.parent);
        }
        chain.iter().rev().fold(Quat::IDENTITY, |rot, n| rot * *n)
    }
}

#[derive(Debug, Error)]
pub enum RawAnimationsError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Gltf(#[from] gltf::Error),
}

#[derive(Default, TypePath)]
pub struct RawAnimationsLoader;

impl AssetLoader for RawAnimationsLoader {
    type Asset = RawAnimations;
    type Error = RawAnimationsError;
    type Settings = ();

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<RawAnimations, RawAnimationsError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let gltf = gltf::Gltf::from_slice(&bytes)?;
        let blob = gltf.blob.as_deref();
        let get_buffer = |buffer: gltf::Buffer| match buffer.source() {
            gltf::buffer::Source::Bin => blob,
            gltf::buffer::Source::Uri(_) => None,
        };

        let mut nodes = HashMap::default();
        let mut parents = HashMap::<String, String>::default();
        for node in gltf.nodes() {
            let Some(name) = node.name() else {
                continue;
            };
            for child in node.children() {
                if let Some(child_name) = child.name() {
                    parents.insert(child_name.to_string(), name.to_string());
                }
            }
        }
        for node in gltf.nodes() {
            let Some(name) = node.name() else {
                continue;
            };
            let (_, rotation, _) = node.transform().decomposed();
            nodes.insert(
                name.to_string(),
                RawNode {
                    rotation: Quat::from_array(rotation),
                    parent:   parents.get(name).cloned(),
                },
            );
        }

        let mut animations = Vec::new();
        for animation in gltf.animations() {
            let mut channels = Vec::new();
            for channel in animation.channels() {
                let Some(target) = channel.target().node().name() else {
                    continue;
                };
                let reader = channel.reader(get_buffer);
                let Some(inputs) = reader.read_inputs() else {
                    continue;
                };
                let Some(ReadOutputs::Rotations(rotations)) = reader.read_outputs() else {
                    continue;
                };
                let values = rotations
                    .into_f32()
                    .map(Quat::from_array)
                    .collect::<Vec<_>>();
                channels.push(RawRotationChannel {
                    target: target.to_string(),
                    timestamps: inputs.collect(),
                    values,
                });
            }
            animations.push(RawAnimation { channels });
        }

        Ok(RawAnimations { animations, nodes })
    }

    fn extensions(&self) -> &[&str] {
        &["glb", "gltf"]
    }
}
