mod common;

pub mod client;
pub mod launcher;

#[cfg_attr(target_os = "linux", path = "platform/linux.rs")]
#[cfg_attr(target_os = "macos", path = "platform/macos.rs")]
#[cfg_attr(windows, path = "platform/windows.rs")]
mod platform;

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
compile_error!("unavi-launcher supports linux, macos and windows only");

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Checking,
    Downloading {
        version:  String,
        progress: Option<f32>,
    },
    UpToDate,
    UpdatedNeedsRestart,
    Offline,
    Error(String),
}
