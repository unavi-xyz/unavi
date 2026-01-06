use std::str::FromStr;

use anyhow::bail;
use blake3::Hash;
use iroh::EndpointId;
use loro::{LoroDoc, LoroValue};
use xdid::core::did::Did;

pub struct Beacon {
    pub did: Did,
    pub endpoint: EndpointId,
    pub expires: i64,
    pub space: Hash,
}

impl Beacon {
    /// Load from a [`LoroDoc`].
    ///
    /// # Errors
    ///
    /// Returns an error if the container is malformed.
    pub fn load(doc: &LoroDoc) -> anyhow::Result<Self> {
        let map = doc.get_map("beacon");
        let value = map.get_deep_value();
        Self::load_from_value(&value)
    }

    /// Load from an already-extracted [`LoroValue`].
    ///
    /// # Errors
    ///
    /// Returns an error if the value is malformed.
    pub fn load_from_value(value: &LoroValue) -> anyhow::Result<Self> {
        let LoroValue::Map(map) = value else {
            anyhow::bail!("value is not a map");
        };

        let LoroValue::String(did_value) = map
            .get("did")
            .ok_or_else(|| anyhow::anyhow!("missing did"))?
        else {
            bail!("invalid did type")
        };
        let did = Did::from_str(did_value.as_str())?;

        let LoroValue::Binary(endpoint_value) = map
            .get("endpoint")
            .ok_or_else(|| anyhow::anyhow!("missing endpoint"))?
        else {
            bail!("invalid endpoint type")
        };
        let endpoint = EndpointId::from_bytes(endpoint_value.as_slice().try_into()?)?;

        let LoroValue::I64(expires) = map
            .get("expires")
            .ok_or_else(|| anyhow::anyhow!("missing expires"))?
        else {
            bail!("invalid expires type")
        };

        let LoroValue::Binary(space_value) = map
            .get("space")
            .ok_or_else(|| anyhow::anyhow!("missing space"))?
        else {
            bail!("invalid space type")
        };
        let space = Hash::from_bytes(space_value.as_slice().try_into()?);

        Ok(Self {
            did,
            endpoint,
            expires: *expires,
            space,
        })
    }

    /// Save to a [`LoroDoc`]'s container.
    ///
    /// # Errors
    ///
    /// Returns an error if the container could not be updated.
    pub fn save(&self, doc: &LoroDoc) -> anyhow::Result<()> {
        let map = doc.get_map("beacon");

        map.insert("did", self.did.to_string())?;
        map.insert("endpoint", self.endpoint.to_vec())?;
        map.insert("expires", self.expires)?;
        map.insert("space", self.space.as_bytes())?;

        Ok(())
    }
}
