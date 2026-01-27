use bevy::prelude::*;

use crate::stage::Attrs;

pub fn parse_xform_attrs(attrs: &Attrs, node: Entity, commands: &mut Commands) {
    if attrs.xform_pos.is_some() || attrs.xform_rot.is_some() || attrs.xform_scale.is_some() {
        let mut transform = Transform::default();

        if let Some(pos) = attrs.xform_pos {
            transform.translation = Vec3::new(pos[0], pos[1], pos[2]);
        }
        if let Some(rot) = attrs.xform_rot {
            transform.rotation = Quat::from_xyzw(rot[0], rot[1], rot[2], rot[3]);
        }
        if let Some(scale) = attrs.xform_scale {
            transform.scale = Vec3::new(scale[0], scale[1], scale[2]);
        }

        commands.entity(node).insert(transform);
    } else {
        commands.entity(node).remove::<Transform>();
    }

    if let Some(_parent) = attrs.xform_parent {
        // TODO
    } else {
        commands.entity(node).remove::<ChildOf>();
    }
}
