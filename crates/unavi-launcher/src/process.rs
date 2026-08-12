use std::{
    process::Child,
    sync::Arc,
};

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

    pub fn set(&self, child: Child) {
        *self.child.lock() = Some(child);
    }

    /// Cleans up the stored child process if it has exited.
    #[must_use]
    pub fn is_running(&self) -> bool {
        let mut guard = self.child.lock();
        if let Some(ref mut child) = *guard {
            if child.try_wait().ok() == Some(None) {
                true
            } else {
                *guard = None;
                false
            }
        } else {
            false
        }
    }

    pub fn kill(&self) -> anyhow::Result<()> {
        let mut guard = self.child.lock();
        if let Some(ref mut child) = *guard {
            child.kill()?;
            *guard = None;
        }
        drop(guard);
        Ok(())
    }
}

impl Default for ProcessTracker {
    fn default() -> Self {
        Self::new()
    }
}
