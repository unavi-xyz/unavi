use std::sync::Arc;

use anyhow::bail;
use bevy::prelude::*;
use hsd::attributes::{Attribute, xform::Xform};
use loro::{ContainerID, Index, LoroDoc, TreeID, event::Diff};

use crate::attributes::{AttrDataEvent, AttributeParser};

pub enum XformEvent {
    SetRotation(Quat),
    SetScale(Vec3),
    SetTranslation(Vec3),
}

pub struct XformParser;

impl AttributeParser for XformParser {
    fn key(&self) -> &'static str {
        Xform::KEY
    }

    fn parse(
        &self,
        doc: &Arc<LoroDoc>,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> anyhow::Result<Option<AttrDataEvent>> {
        if path.is_empty() {
            return Ok(None);
        }

        let key = path[0]
            .1
            .as_key()
            .ok_or_else(|| anyhow::anyhow!("invalid index type"))?;

        let val = diff
            .as_list()
            .ok_or_else(|| anyhow::anyhow!("invalid diff type"))?;

        match key.as_str() {
            "rotation" => Ok(Some(AttrDataEvent::Xform(XformEvent::SetRotation(
                todo!(), // Quat::from_slice(slice),
            )))),
            "scale" => Ok(None),
            "translation" => Ok(None),
            _ => bail!("unknown key"),
        }
    }
}
