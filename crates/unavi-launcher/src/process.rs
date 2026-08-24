use std::process::Child;

use parking_lot::Mutex;

pub struct ProcessTracker {
    child: Mutex<Option<Child>>,
}

impl ProcessTracker {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            child: Mutex::new(None),
        }
    }

    pub fn set(&self, child: Child) {
        *self.child.lock() = Some(child);
    }

    /// Cleans up the stored child process if it has exited.
    #[must_use]
    pub fn is_running(&self) -> bool {
        let mut guard = self.child.lock();

        let running = guard
            .as_mut()
            .is_some_and(|child| matches!(child.try_wait(), Ok(None)));

        if !running {
            *guard = None;
        }

        running
    }

    pub fn kill(&self) -> anyhow::Result<()> {
        let child = self.child.lock().take();

        if let Some(mut child) = child {
            child.kill()?;
            // Reaps the process; dropping a `Child` deliberately does not.
            child.wait()?;
        }

        Ok(())
    }
}

impl Default for ProcessTracker {
    fn default() -> Self {
        Self::new()
    }
}
