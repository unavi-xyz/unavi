use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RecordLink {
    pub did: String,
    pub record_id: String,
}
