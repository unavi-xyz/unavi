pub mod client;
pub mod common;
pub mod launcher;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Checking,
    Downloading {
        version: String,
        progress: Option<f32>,
    },
    UpToDate,
    UpdatedNeedsRestart,
    Offline,
    Error(String),
}
