use std::{
    process::Child,
    sync::{Arc, Mutex},
};

/// Tracks a spawned client process
#[derive(Clone)]
pub struct ProcessTracker {
    child: Arc<Mutex<Option<Child>>>,
}

impl ProcessTracker {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the tracked process
    pub fn set(&self, child: Child) {
        *self.child.lock().unwrap() = Some(child);
    }

    /// Check if the process is still running
    pub fn is_running(&self) -> bool {
        let mut guard = self.child.lock().unwrap();
        if let Some(ref mut child) = *guard {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    *guard = None;
                    false
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(_) => {
                    // Error checking status, assume not running
                    *guard = None;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Try to kill the process if it's running
    pub fn kill(&self) -> anyhow::Result<()> {
        let mut guard = self.child.lock().unwrap();
        if let Some(ref mut child) = *guard {
            child.kill()?;
            *guard = None;
        }
        Ok(())
    }
}

impl Default for ProcessTracker {
    fn default() -> Self {
        Self::new()
    }
}
