use bevy::{
    ecs::component::Mutable,
    prelude::*,
};
use hsd::{
    attributes,
    property::{
        Payload,
        name::PropName,
    },
};

mod buffer;
pub mod collider;
mod gravity_scale;
pub mod image;
pub mod material;
pub(crate) mod mesh;
mod name;
pub mod portal;
mod rigid_body;
pub mod script;
pub mod shader;
pub mod spawn;
mod text;
mod values;
pub(crate) mod xform;

/// Inserts what `build` makes of the decoded payload. Removes `B` on removal
/// or when `build` returns `None`.
pub(crate) fn apply_simple<A, B>(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
    build: impl FnOnce(A) -> Option<B>,
) -> Result<(), postcard::Error>
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

/// Mutates `T` on `prim`, defaulting it first if absent.
pub(crate) fn update_data<T: Component<Mutability = Mutable> + Default>(
    commands: &mut Commands,
    prim: Entity,
    f: impl FnOnce(&mut T) + Send + Sync + 'static,
) {
    commands
        .entity(prim)
        .entry::<T>()
        .or_default()
        .and_modify(move |mut data| f(&mut data));
}

/// Applies one attribute field to a prim. A `None` payload is a removal.
/// Unknown groups are skipped.
pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    match name.group() {
        attributes::collider::GROUP => collider::apply(commands, prim, name, payload),
        attributes::gravity_scale::GROUP => gravity_scale::apply(commands, prim, payload),
        attributes::image::GROUP => image::apply(commands, prim, name, payload),
        attributes::material::GROUP => material::apply(commands, prim, name, payload),
        attributes::mesh::GROUP => mesh::apply(commands, prim, name, payload),
        attributes::name::GROUP => name::apply(commands, prim, payload),
        attributes::portal::GROUP => portal::apply(commands, prim, payload),
        attributes::reference::GROUP => crate::reference::apply(commands, prim, payload),
        attributes::rigid_body::GROUP => rigid_body::apply(commands, prim, payload),
        attributes::script::GROUP => script::apply(commands, prim, payload),
        attributes::shader::GROUP => shader::apply(commands, prim, name, payload),
        attributes::spawn::GROUP => spawn::apply(commands, prim, payload),
        attributes::text::GROUP => text::apply(commands, prim, payload),
        attributes::xform::GROUP => xform::apply(commands, prim, payload),
        _ => Ok(()),
    }
}
