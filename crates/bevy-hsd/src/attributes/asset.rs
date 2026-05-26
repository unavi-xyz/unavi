use bevy::prelude::*;
use hsd::attributes::{
    Attribute,
    asset::AssetAttr,
};
use loro::{
    LoroValue,
    ValueOrContainer,
};

use crate::attributes::{
    AttributeParser,
    ParseError,
};

#[derive(Component, Debug, Clone)]
pub struct HsdAsset(pub blake3::Hash);

pub struct AssetParser;

impl AttributeParser for AssetParser {
    fn key(&self) -> &'static str {
        AssetAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        match value {
            Some(ValueOrContainer::Value(LoroValue::Binary(bytes))) => {
                if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                    commands
                        .entity(prim)
                        .insert(HsdAsset(blake3::Hash::from_bytes(arr)));
                } else {
                    warn!("asset attribute: expected 32-byte blob id");
                }
            }
            Some(other) => {
                warn!(?other, "asset attribute must be a binary blob id");
            }
            None => {
                commands.entity(prim).remove::<HsdAsset>();
            }
        }
        Ok(())
    }
}
