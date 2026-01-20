use loro::LoroDoc;
use loro_surgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

use crate::conv;

/// Access control list for a record.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Hydrate, Reconcile)]
pub struct Acl {
    #[loro(default)]
    pub public: bool,
    #[loro(with = "conv::did::list", default)]
    pub manage: Vec<Did>,
    #[loro(with = "conv::did::list", default)]
    pub read: Vec<Did>,
    #[loro(with = "conv::did::list", default)]
    pub write: Vec<Did>,
}

impl Acl {
    #[must_use]
    pub const fn is_public(&self) -> bool {
        self.public
    }

    /// Returns `true` if public, in `read` list, or has write/manage permission.
    #[must_use]
    pub fn can_read(&self, did: &Did) -> bool {
        self.public || self.read.contains(did) || self.can_write(did)
    }

    #[must_use]
    pub fn can_write(&self, did: &Did) -> bool {
        self.write.contains(did) || self.can_manage(did)
    }

    #[must_use]
    pub fn can_manage(&self, did: &Did) -> bool {
        self.manage.contains(did)
    }

    /// # Errors
    ///
    /// Returns an error if the ACL could not be saved.
    pub fn save(&self, doc: &LoroDoc) -> anyhow::Result<()> {
        let map = doc.get_map("acl");
        self.reconcile(&map)?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if the ACL container is malformed.
    pub fn load(doc: &LoroDoc) -> anyhow::Result<Self> {
        let map = doc.get_map("acl");
        let value = map.get_deep_value();
        Self::hydrate(&value).map_err(|e| anyhow::anyhow!("{e}"))
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
