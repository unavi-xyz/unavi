//! Access control list type for WDS documents.

use loro::LoroDoc;
use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError, loro::LoroMap};
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

use crate::conv;

/// Access control list for a record.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Acl {
    /// Whether the record is publicly readable.
    pub public: bool,
    /// DIDs with manage permission (can modify ACL).
    pub manage: Vec<Did>,
    /// DIDs with write permission.
    pub write: Vec<Did>,
    /// DIDs with read permission.
    pub read: Vec<Did>,
}

impl Acl {
    /// Check if this record is public.
    #[must_use]
    pub const fn is_public(&self) -> bool {
        self.public
    }

    /// Check if a DID has read permission.
    ///
    /// Returns `true` if:
    /// - The record is public
    /// - The DID is in the `read` list
    /// - The DID has write or manage permission
    #[must_use]
    pub fn can_read(&self, did: &Did) -> bool {
        self.public || self.read.contains(did) || self.can_write(did)
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

    /// Save ACL to a [`LoroDoc`]'s "acl" container.
    ///
    /// # Errors
    ///
    /// Returns an error if the ACL could not be saved.
    pub fn save(&self, doc: &LoroDoc) -> anyhow::Result<()> {
        let map = doc.get_map("acl");
        self.reconcile(&map)?;
        Ok(())
    }

    /// Load ACL from a [`LoroDoc`]'s "acl" container.
    ///
    /// # Errors
    ///
    /// Returns an error if the ACL container is malformed.
    pub fn load(doc: &LoroDoc) -> anyhow::Result<Self> {
        let map = doc.get_map("acl");
        let value = map.get_deep_value();
        Self::hydrate(&value).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl Hydrate for Acl {
    fn hydrate(value: &loro::LoroValue) -> Result<Self, HydrateError> {
        let loro::LoroValue::Map(map) = value else {
            return Err(HydrateError::TypeMismatch {
                path: String::new(),
                expected: "map".into(),
                actual: format!("{value:?}"),
            });
        };

        let public = map
            .get("public")
            .map(|v| match v {
                loro::LoroValue::Bool(b) => Ok(*b),
                _ => Err(HydrateError::TypeMismatch {
                    path: "public".into(),
                    expected: "bool".into(),
                    actual: format!("{v:?}"),
                }),
            })
            .transpose()?
            .unwrap_or(false);

        let manage = map
            .get("manage")
            .map(conv::did::list::hydrate)
            .transpose()?
            .unwrap_or_default();

        let write = map
            .get("write")
            .map(conv::did::list::hydrate)
            .transpose()?
            .unwrap_or_default();

        let read = map
            .get("read")
            .map(conv::did::list::hydrate)
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            public,
            manage,
            write,
            read,
        })
    }
}

impl Reconcile for Acl {
    fn reconcile(&self, map: &LoroMap) -> Result<(), ReconcileError> {
        map.insert("public", self.public)?;
        conv::did::list::reconcile_field(&self.manage, map, "manage")?;
        conv::did::list::reconcile_field(&self.write, map, "write")?;
        conv::did::list::reconcile_field(&self.read, map, "read")?;
        Ok(())
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let nested = map.get_or_create_container(key, loro::LoroMap::new())?;
        self.reconcile(&nested)
    }
}

#[cfg(test)]
mod tests {
    use loro::LoroDoc;
    use rstest::rstest;
    use xdid::core::did::Did;

    use super::*;

    fn test_did() -> Did {
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
            .parse()
            .expect("valid did")
    }

    #[rstest]
    fn roundtrip_acl_default() {
        let doc = LoroDoc::new();
        let acl = Acl::default();

        acl.save(&doc).expect("save failed");
        let loaded = Acl::load(&doc).expect("load failed");

        assert!(!loaded.public);
        assert!(loaded.manage.is_empty());
        assert!(loaded.write.is_empty());
        assert!(loaded.read.is_empty());
    }

    #[rstest]
    fn roundtrip_acl_with_permissions() {
        let doc = LoroDoc::new();
        let did = test_did();
        let acl = Acl {
            public: true,
            manage: vec![did.clone()],
            write: vec![did.clone()],
            read: vec![did.clone()],
        };

        acl.save(&doc).expect("save failed");
        let loaded = Acl::load(&doc).expect("load failed");

        assert!(loaded.public);
        assert_eq!(loaded.manage.len(), 1);
        assert_eq!(loaded.write.len(), 1);
        assert_eq!(loaded.read.len(), 1);
        assert!(loaded.can_manage(&did));
        assert!(loaded.can_write(&did));
        assert!(loaded.can_read(&did));
    }
}
