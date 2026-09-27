use std::marker::PhantomData;

use bevy::prelude::*;
use hsd::{
    property::{
        Payload,
        name::PropName,
    },
    schema,
};
use thiserror::Error;

pub mod collider;
mod color;
mod gravity_scale;
pub mod image;
pub mod material;
pub(crate) mod material_source;
pub(crate) mod mesh;
mod name;
pub mod portal;
pub(crate) mod reference;
mod relations;
mod rigid_body;
pub mod script;
pub mod shader;
pub mod spawn;
mod text;
pub(crate) mod util;
pub(crate) mod xform;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("postcard {0}")]
    Postcard(#[from] postcard::Error),
}

/// Marks a prim whose `T` has not been built from its latest data. The rebuild
/// system removes it after every attempt, including a failed one.
#[derive(Component)]
pub(crate) struct Pending<T>(PhantomData<T>);

impl<T> Default for Pending<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Decodes `payload` as `A` and hands it to `build`; inserts the resulting
/// bundle, or removes `B` on removal or when `build` finds the value invalid.
pub(crate) fn apply_simple<A, B>(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
    build: impl FnOnce(A) -> Option<B>,
) -> Result<(), ParseError>
where
    A: Payload,
    B: Bundle,
{
    match payload {
        Some(bytes) => {
            let attr = A::decode(bytes)?;
            match build(attr) {
                Some(bundle) => commands.entity(prim).insert(bundle),
                None => commands.entity(prim).remove::<B>(),
            };
        }
        None => {
            commands.entity(prim).remove::<B>();
        }
    }
    Ok(())
}

/// Decodes the field's payload and applies it to the prim, keyed by group:
/// every property under one group is one attribute's fields, so one handler
/// covers the field it is called for.
///
/// `None` payload means the field was removed. A group this build has never
/// heard of has no handler and is skipped — its entries still store, sync and
/// re-serve untouched.
pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    match name.group() {
        schema::collider::GROUP => collider::apply(commands, prim, name, payload),
        schema::gravity_scale::GROUP => gravity_scale::apply(commands, prim, payload),
        schema::image::GROUP => image::apply(commands, prim, name, payload),
        schema::material::GROUP => material::apply(commands, prim, name, payload),
        schema::mesh::GROUP => mesh::apply(commands, prim, name, payload),
        schema::name::GROUP => name::apply(commands, prim, payload),
        schema::portal::GROUP => portal::apply(commands, prim, payload),
        schema::reference::GROUP => reference::apply(commands, prim, payload),
        schema::rigid_body::GROUP => rigid_body::apply(commands, prim, payload),
        schema::script::GROUP => script::apply(commands, prim, payload),
        schema::shader::GROUP => shader::apply(commands, prim, name, payload),
        schema::spawn::GROUP => spawn::apply(commands, prim, payload),
        schema::text::GROUP => text::apply(commands, prim, payload),
        schema::xform::GROUP => xform::apply(commands, prim, payload),
        _ => Ok(()),
    }
}
