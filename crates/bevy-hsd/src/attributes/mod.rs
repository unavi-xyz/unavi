use std::sync::LazyLock;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use thiserror::Error;

pub mod collider;
pub mod gravity_scale;
pub mod image;
pub mod material;
pub mod material_graph;
pub mod material_source;
pub mod mesh;
pub mod name;
pub mod portal;
pub mod prefab;
pub mod rigid_body;
pub mod script;
pub mod spawn;
pub mod util;
pub mod xform;

pub static PARSERS: LazyLock<HashMap<&'static str, Box<dyn AttributeParser>>> =
    LazyLock::new(|| {
        let parsers: [Box<dyn AttributeParser>; _] = [
            Box::new(collider::ColliderParser),
            Box::new(gravity_scale::GravityScaleParser),
            Box::new(image::ImageParser),
            Box::new(material::MaterialParser),
            Box::new(material_graph::ShaderGraphOverridesParser),
            Box::new(mesh::MeshParser),
            Box::new(name::NameParser),
            Box::new(portal::PortalParser),
            Box::new(rigid_body::RigidBodyParser),
            Box::new(spawn::SpawnParser),
            Box::new(xform::XformParser),
        ];
        let mut map = HashMap::default();
        for attr in parsers {
            map.insert(attr.key(), attr);
        }
        map
    });

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("postcard {0}")]
    Postcard(#[from] postcard::Error),
}

/// One hook per attribute: decode the payload and put the result on the prim.
///
/// An attribute this build has never heard of has no parser and is skipped —
/// its entry still stores, syncs and re-serves untouched.
pub trait AttributeParser: Send + Sync {
    fn key(&self) -> &'static str;

    /// `None` means the property was removed.
    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        payload: Option<&[u8]>,
    ) -> Result<(), ParseError>;
}
