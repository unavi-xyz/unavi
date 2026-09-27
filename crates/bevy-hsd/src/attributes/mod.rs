use std::sync::LazyLock;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use hsd::property::name::PropName;
use thiserror::Error;

pub mod collider;
pub mod gravity_scale;
pub mod image;
pub mod material;
pub mod material_source;
pub mod mesh;
pub mod name;
pub mod portal;
pub mod reference;
pub mod rigid_body;
pub mod script;
pub mod shader;
pub mod spawn;
pub mod text;
pub mod util;
pub mod xform;

/// Keyed by group: every property under one group is one attribute's
/// fields, so one parser dispatches on the field it is called for.
pub static PARSERS: LazyLock<HashMap<&'static str, Box<dyn AttributeParser>>> =
    LazyLock::new(|| {
        let parsers: [Box<dyn AttributeParser>; _] = [
            Box::new(collider::ColliderParser),
            Box::new(gravity_scale::GravityScaleParser),
            Box::new(image::ImageParser),
            Box::new(material::MaterialParser),
            Box::new(mesh::MeshParser),
            Box::new(name::NameParser),
            Box::new(portal::PortalParser),
            Box::new(reference::ReferenceParser),
            Box::new(rigid_body::RigidBodyParser),
            Box::new(script::ScriptParser),
            Box::new(shader::ShaderParser),
            Box::new(spawn::SpawnParser),
            Box::new(text::TextParser),
            Box::new(xform::XformParser),
        ];
        let mut map = HashMap::default();
        for attr in parsers {
            map.insert(attr.group(), attr);
        }
        map
    });

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("postcard {0}")]
    Postcard(#[from] postcard::Error),
}

/// One hook per group: decode the field's payload and put the result on
/// the prim.
///
/// A group this build has never heard of has no parser and is skipped —
/// its entries still store, sync and re-serve untouched.
pub trait AttributeParser: Send + Sync {
    fn group(&self) -> &'static str;

    /// `None` means the field was removed.
    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        name: &PropName,
        payload: Option<&[u8]>,
    ) -> Result<(), ParseError>;
}
