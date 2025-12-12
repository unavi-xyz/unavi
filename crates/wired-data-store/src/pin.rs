use crate::RecordId;

/// A pin represents an explicit subscription to a record.
pub struct Pin {
    pub record_id: RecordId,
    pub created: u64,
    pub expires: Option<u64>,
}
