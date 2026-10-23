//! Application-defined properties: any `group/field` outside the groups the
//! format defines.

use crate::{
    attributes::{
        collider,
        gravity_scale,
        image,
        material,
        mesh,
        name,
        parent,
        portal,
        reference,
        rigid_body,
        script,
        shader,
        sky,
        spawn,
        text,
        xform,
    },
    property::name::PropName,
};

/// Largest value one custom property may hold.
pub const MAX_CUSTOM_BYTES: usize = 64 * 1024;

/// Every group the format defines. A custom property must live outside them,
/// so raw bytes never land in a field a reader decodes.
const BUILTIN_GROUPS: [&str; 16] = [
    collider::GROUP,
    gravity_scale::GROUP,
    image::GROUP,
    material::GROUP,
    mesh::GROUP,
    name::GROUP,
    parent::GROUP,
    portal::GROUP,
    reference::GROUP,
    rigid_body::GROUP,
    script::GROUP,
    shader::GROUP,
    sky::GROUP,
    spawn::GROUP,
    text::GROUP,
    xform::GROUP,
];

/// Whether `name` is a `group/field` outside every built-in group.
#[must_use]
pub fn is_custom(name: &PropName) -> bool {
    name.field().is_some() && !BUILTIN_GROUPS.contains(&name.group())
}

/// Parses an application-defined `group/field` name, refusing a bare group or
/// one the format defines.
pub fn custom_name(name: &str) -> Result<PropName, &'static str> {
    let parsed = name
        .parse::<PropName>()
        .map_err(|_| "a custom name is `group/field`, at most 1 KiB")?;
    if parsed.field().is_none() {
        return Err("a custom name is `group/field`");
    }
    if BUILTIN_GROUPS.contains(&parsed.group()) {
        return Err("a custom name must not use a group the host defines");
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_custom_name_needs_a_field_and_a_free_group() {
        assert!(custom_name("edit/locked").is_ok());
        assert!(custom_name("edit").is_err());
        assert!(custom_name("name/x").is_err());
    }
}
