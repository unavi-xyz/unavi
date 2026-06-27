use bevy::prelude::*;
use hsd::attributes::{
    Attribute,
    subdocument::SubdocumentAttr,
};
use loro::ValueOrContainer;

use crate::attributes::{
    AttributeParser,
    ParseError,
};

/// Marks a prim that declares a child sub-document. The instancing layer reads
/// the prim's `subdocument` attribute off the doc to materialize or instance
/// it.
#[derive(Component)]
pub struct HsdSubdocument;

pub struct SubdocumentParser;

impl AttributeParser for SubdocumentParser {
    fn key(&self) -> &'static str {
        SubdocumentAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        if value.is_some() {
            commands.entity(prim).insert(HsdSubdocument);
        } else {
            commands.entity(prim).remove::<HsdSubdocument>();
        }
        Ok(())
    }
}
