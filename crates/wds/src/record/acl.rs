use std::str::FromStr;

use loro::{LoroDoc, LoroValue};
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

/// Access control list for a record.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Acl {
    pub manage: Vec<Did>,
    pub write: Vec<Did>,
    pub read: Vec<Did>,
}

impl Acl {
    /// Load ACL from a [`LoroDoc`]'s "acl" container.
    ///
    /// # Errors
    ///
    /// Returns an error if the ACL container is malformed.
    pub fn load(doc: &LoroDoc) -> anyhow::Result<Self> {
        let map = doc.get_map("acl");
        let value = map.get_deep_value();
        Self::load_from_value(&value)
    }

    /// Load ACL from an already-extracted [`LoroValue`].
    ///
    /// # Errors
    ///
    /// Returns an error if the value is malformed.
    pub fn load_from_value(value: &LoroValue) -> anyhow::Result<Self> {
        let LoroValue::Map(map) = value else {
            anyhow::bail!("acl is not a map");
        };

        let manage = extract_did_list(map, "manage")?;
        let write = extract_did_list(map, "write")?;
        let read = extract_did_list(map, "read")?;

        Ok(Self {
            manage,
            write,
            read,
        })
    }

    /// Save ACL to a [`LoroDoc`]'s "acl" container.
    ///
    /// # Errors
    ///
    /// Returns an error if the ACL container could not be updated.
    pub fn save(&self, doc: &LoroDoc) -> anyhow::Result<()> {
        let map = doc.get_map("acl");

        let manage = self.manage.iter().map(Did::to_string).collect::<Vec<_>>();
        map.insert("manage", manage)?;

        let write = self.write.iter().map(Did::to_string).collect::<Vec<_>>();
        map.insert("write", write)?;

        let read = self.read.iter().map(Did::to_string).collect::<Vec<_>>();
        map.insert("read", read)?;

        Ok(())
    }

    /// Check if a DID has read permission (read, write, or manage).
    #[must_use]
    pub fn can_read(&self, did: &Did) -> bool {
        self.read.contains(did) || self.can_write(did)
    }

    /// Check if a DID has write permission (write or manage).
    #[must_use]
    pub fn can_write(&self, did: &Did) -> bool {
        self.write.contains(did) || self.can_manage(did)
    }

    /// Check if a DID has manage permission.
    #[must_use]
    pub fn can_manage(&self, did: &Did) -> bool {
        self.manage.contains(did)
    }
}

/// Extract a list of DIDs from a map field.
fn extract_did_list(map: &loro::LoroMapValue, key: &str) -> anyhow::Result<Vec<Did>> {
    let Some(value) = map.get(key) else {
        return Ok(Vec::new());
    };

    let LoroValue::List(list) = value else {
        anyhow::bail!("acl.{key} is not a list");
    };

    list.iter()
        .map(|v| {
            let LoroValue::String(s) = v else {
                anyhow::bail!("acl.{key} contains non-string value");
            };
            Did::from_str(s).map_err(|e| anyhow::anyhow!("invalid DID in acl.{key}: {e}"))
        })
        .collect()
}
