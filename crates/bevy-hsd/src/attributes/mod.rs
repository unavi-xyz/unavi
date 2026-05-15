use std::sync::{Arc, LazyLock, mpsc::SendError};

use bevy::{platform::collections::HashMap, prelude::*};
use loro::{ContainerID, Index, LoroDoc, LoroError, TreeID, ValueOrContainer, event::Diff};
use lorosurgeon::HydrateError;
use thiserror::Error;

use crate::diff::{DiffSender, HsdDiffEvent};

pub mod image;
pub mod mesh;
pub mod name;
mod util;
pub mod xform;

pub static PARSERS: LazyLock<HashMap<&'static str, Box<dyn AttributeParser>>> =
    LazyLock::new(|| {
        let parsers: [Box<dyn AttributeParser>; _] = [
            Box::new(image::ImageParser),
            Box::new(mesh::MeshParser),
            Box::new(name::NameParser),
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
    Image(image::ImageEvent),
    Mesh(mesh::MeshEvent),
    Name(name::NameEvent),
    Xform(xform::XformEvent),
}

#[derive(Clone)]
pub struct DocContext {
    pub doc: Arc<LoroDoc>,
    pub tx: DiffSender,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("loro {0}")]
    Loro(#[from] LoroError),
    #[error("hydrate {0}")]
    Hydrate(#[from] HydrateError),
    #[error("failed to send diff event")]
    Send(#[from] SendError<HsdDiffEvent>),
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

    fn parse(
        &self,
        ctx: &DocContext,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> Result<(), ParseError>;
}

#[derive(EntityEvent)]
pub struct ApplyEvent<T> {
    pub entity: Entity,
    pub value: T,
}
