#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Checking,
    Downloading(String),
    UpToDate,
    UpdatedNeedsRestart,
    Error(String),
}
