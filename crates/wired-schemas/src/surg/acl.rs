use loro::LoroDoc;
use loro_surgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

use crate::HydratedDid;

/// Access control list for a record.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Hydrate, Reconcile)]
pub struct Acl {
    #[loro(default)]
    pub public: bool,
    #[loro(default)]
    manage: Vec<HydratedDid>,
    #[loro(default)]
    read: Vec<HydratedDid>,
    #[loro(default)]
    write: Vec<HydratedDid>,
}

impl Acl {
    #[must_use]
    pub fn managers(&self) -> &[HydratedDid] {
        &self.manage
    }
    pub fn add_manager(&mut self, did: Did) {
        self.manage.push(HydratedDid(did));
    }
    pub fn remove_manager(&mut self, did: &Did) {
        self.manage.retain(|d| &d.0 != did);
    }

    #[must_use]
    pub fn readers(&self) -> &[HydratedDid] {
        &self.read
    }
    pub fn add_reader(&mut self, did: Did) {
        self.read.push(HydratedDid(did));
    }
    pub fn remove_reader(&mut self, did: &Did) {
        self.read.retain(|d| &d.0 != did);
    }

    #[must_use]
    pub fn writers(&self) -> &[HydratedDid] {
        &self.write
    }
    pub fn add_writer(&mut self, did: Did) {
        self.write.push(HydratedDid(did));
    }
    pub fn remove_writer(&mut self, did: &Did) {
        self.write.retain(|d| &d.0 != did);
    }

    #[must_use]
    pub fn can_read(&self, did: &Did) -> bool {
        self.public || self.read.iter().any(|d| &d.0 == did)
    }

    #[must_use]
    pub fn can_write(&self, did: &Did) -> bool {
        self.write.iter().any(|d| &d.0 == did)
    }

    #[must_use]
    pub fn can_manage(&self, did: &Did) -> bool {
        self.manage.iter().any(|d| &d.0 == did)
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

    use super::*;

    fn test_did() -> HydratedDid {
        HydratedDid(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
                .parse()
                .expect("valid did"),
        )
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
        let wdid = test_did();
        let did = &wdid.0;
        let acl = Acl {
            public: true,
            manage: vec![wdid.clone()],
            write: vec![wdid.clone()],
            read: vec![wdid.clone()],
        };

        acl.save(&doc).expect("save failed");
        let loaded = Acl::load(&doc).expect("load failed");

        assert!(loaded.public);
        assert_eq!(loaded.manage.len(), 1);
        assert_eq!(loaded.write.len(), 1);
        assert_eq!(loaded.read.len(), 1);
        assert!(loaded.can_manage(did));
        assert!(loaded.can_write(did));
        assert!(loaded.can_read(did));
    }
}
