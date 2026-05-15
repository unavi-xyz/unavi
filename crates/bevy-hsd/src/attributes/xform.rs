use std::sync::Arc;

use anyhow::bail;
use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{Attribute, xform::Xform},
};
use loro::{ContainerID, Index, LoroDoc, TreeID, ValueOrContainer, event::Diff};

use crate::attributes::{ApplyEvent, AttrDataEvent, AttributeParser};

pub enum XformEvent {
    Rotation(Quat),
    Scale(Vec3),
    Translation(Vec3),
}

pub struct XformParser;

impl AttributeParser for XformParser {
    fn key(&self) -> &'static str {
        Xform::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> anyhow::Result<()> {
        if value.is_some() {
            commands.entity(prim).insert(Transform::default());
        } else {
            commands.entity(prim).remove::<Transform>();
        }
        Ok(())
    }

    fn parse(
        &self,
        doc: &Arc<LoroDoc>,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        _diff: Diff,
    ) -> anyhow::Result<Option<AttrDataEvent>> {
        if path.is_empty() {
            return Ok(None);
        }

        let key = path[0]
            .1
            .as_key()
            .ok_or_else(|| anyhow::anyhow!("invalid index type"))?;

        let tree = doc.get_tree(&*HSD_CONTAINER_ID);
        let meta = tree.get_meta(prim)?;

        let xform = Xform::attr_hydrate(&meta)?;

        match key.as_str() {
            "rotation" => Ok(Some(AttrDataEvent::Xform(XformEvent::Rotation(
                Quat::from_slice(&xform.rotation),
            )))),
            "scale" => Ok(Some(AttrDataEvent::Xform(XformEvent::Scale(
                Vec3::from_slice(&xform.rotation),
            )))),
            "translation" => Ok(Some(AttrDataEvent::Xform(XformEvent::Translation(
                Vec3::from_slice(&xform.rotation),
            )))),
            _ => bail!("unknown key"),
        }
    }
}

pub fn apply_xform(trigger: On<ApplyEvent<XformEvent>>, mut xforms: Query<&mut Transform>) {
    let Ok(mut xform) = xforms.get_mut(trigger.entity) else {
        warn!("Transform not found");
        return;
    };

    match trigger.value {
        XformEvent::Rotation(v) => xform.rotation = v,
        XformEvent::Scale(v) => xform.scale = v,
        XformEvent::Translation(v) => xform.translation = v,
    }
}
