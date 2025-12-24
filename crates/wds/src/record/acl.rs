use loro::LoroDoc;
use serde::{Deserialize, Serialize};
use xdid::core::did::Did;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Acl {
    pub manager: Vec<Did>,
    pub writer: Vec<Did>,
    pub reader: Vec<Did>,
}

impl Acl {
    pub fn save(&self, doc: &LoroDoc) -> anyhow::Result<()> {
        let map = doc.get_map("acl");

        let manager = self.manager.iter().map(Did::to_string).collect::<Vec<_>>();
        map.insert("manager", manager)?;

        let writer = self.writer.iter().map(Did::to_string).collect::<Vec<_>>();
        map.insert("writer", writer)?;

        let reader = self.reader.iter().map(Did::to_string).collect::<Vec<_>>();
        map.insert("reader", reader)?;

        Ok(())
    }
}
