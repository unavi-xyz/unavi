use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{Attribute, hydrate_attr, xform::XformAttr},
};
use loro::{ContainerID, Index, TreeID, ValueOrContainer, event::Diff};

use crate::{
    attributes::{
        ApplyEvent, AttrDataEvent, AttributeParser, DocContext, ParseError,
        util::shallow_map_updated_keys,
    },
    diff::HsdDiffEvent,
};

#[derive(Debug)]
pub enum XformEvent {
    Rotation(Quat),
    Scale(Vec3),
    Translation(Vec3),
}

pub struct XformParser;

impl AttributeParser for XformParser {
    fn key(&self) -> &'static str {
        XformAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        if value.is_some() {
            commands.entity(prim).insert(Transform::default());
        } else {
            commands.entity(prim).remove::<Transform>();
        }
        Ok(())
    }

    fn parse(
        &self,
        ctx: &DocContext,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> Result<(), ParseError> {
        let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
        let meta = tree.get_meta(prim)?;

        let attr: XformAttr = hydrate_attr(&meta)?;

        let keys = shallow_map_updated_keys(path, diff)?;
        for key in keys {
            let event = match key.as_str() {
                "rotation" => XformEvent::Rotation(Quat::from_slice(&attr.rotation)),
                "scale" => XformEvent::Scale(Vec3::from_slice(&attr.scale)),
                "translation" => XformEvent::Translation(Vec3::from_slice(&attr.translation)),
                _ => continue,
            };
            ctx.tx.send(HsdDiffEvent::AttrData {
                prim,
                data: AttrDataEvent::Xform(event),
            })?;
        }
        Ok(())
    }
}

pub fn apply_xform(trigger: On<ApplyEvent<XformEvent>>, mut xforms: Query<&mut Transform>) {
    let Ok(mut transform) = xforms.get_mut(trigger.entity) else {
        warn!("Transform not found");
        return;
    };

    match trigger.value {
        XformEvent::Rotation(v) => transform.rotation = v,
        XformEvent::Scale(v) => transform.scale = v,
        XformEvent::Translation(v) => transform.translation = v,
    }
}
