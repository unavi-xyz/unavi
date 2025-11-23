use std::{process::Child, sync::Arc};

use parking_lot::Mutex;

pub struct ProcessTracker {
    child: Arc<Mutex<Option<Child>>>,
}

impl ProcessTracker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the tracked child process.
    pub fn set(&self, child: Child) {
        *self.child.lock() = Some(child);
    }

    /// Check if the tracked process is still running.
    ///
    /// This method also cleans up the stored child process if it has exited.
    #[must_use]
    pub fn is_running(&self) -> bool {
        let mut guard = self.child.lock();
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

    /// Kill the tracked process if it exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the process cannot be killed.
    pub fn kill(&self) -> anyhow::Result<()> {
        let mut guard = self.child.lock();
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
