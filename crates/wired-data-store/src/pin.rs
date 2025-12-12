use crate::RecordId;

/// Unique identifier for a pin subscription.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PinId(pub u64);

/// A pin represents an explicit subscription to a record.
pub struct Pin {
    pub id: PinId,
    pub record_id: RecordId,
    pub created: u64,
}
