use std::sync::{
    Arc,
    LazyLock,
};

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use loro::{
    ContainerID,
    Index,
    LoroDoc,
    LoroError,
    TreeID,
    ValueOrContainer,
    event::Diff,
};
use loro_surgeon::error::HydrateError;
use thiserror::Error;

use crate::diff::DiffSender;

pub mod asset;
pub mod collider;
pub mod image;
pub mod material;
pub mod mesh;
pub mod name;
pub mod rigid_body;
pub mod script;
pub mod util;
pub mod xform;

pub static PARSERS: LazyLock<HashMap<&'static str, Box<dyn AttributeParser>>> =
    LazyLock::new(|| {
        let parsers: [Box<dyn AttributeParser>; _] = [
            Box::new(asset::AssetParser),
            Box::new(collider::ColliderParser),
            Box::new(image::ImageParser),
            Box::new(material::MaterialParser),
            Box::new(mesh::MeshParser),
            Box::new(name::NameParser),
            Box::new(rigid_body::RigidBodyParser),
            Box::new(script::ScriptParser),
            Box::new(xform::XformParser),
        ];
        let mut map = HashMap::default();
        for attr in parsers {
            map.insert(attr.key(), attr);
        }
        map
    });

#[derive(Debug)]
pub enum AttrDataEvent {
    Collider(collider::ColliderEvent),
    Image(image::ImageEvent),
    Material(material::MaterialEvent),
    Mesh(mesh::MeshEvent),
    RigidBody(rigid_body::RigidBodyEvent),
    Xform(xform::XformEvent),
}

#[derive(Clone)]
pub struct DocContext {
    pub doc: Arc<LoroDoc>,
    pub tx:  DiffSender,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("loro {0}")]
    Loro(#[from] LoroError),
    #[error("hydrate {0}")]
    Hydrate(#[from] HydrateError),
    #[error("failed to send diff event")]
    SendDiff,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub trait AttributeParser: Send + Sync {
    fn key(&self) -> &'static str;

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError>;

    /// Called when an attribute's inner data changes. Scalar attributes
    /// (whose entire value is delivered via [`Self::lifecycle`]) can leave
    /// this as the default no-op.
    fn parse(
        &self,
        _ctx: &DocContext,
        _prim: TreeID,
        _path: &[(ContainerID, Index)],
        _diff: Diff,
    ) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(EntityEvent)]
pub struct ApplyEvent<T> {
    pub entity: Entity,
    pub value:  T,
}
