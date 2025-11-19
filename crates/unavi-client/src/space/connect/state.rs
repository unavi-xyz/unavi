use std::time::{Duration, Instant};

use bevy::prelude::*;

/// Connection state for a space.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected, ready to initiate connection.
    #[default]
    Disconnected,
    /// Actively attempting to connect.
    Connecting { attempt: u32 },
    /// Successfully connected and joined space.
    Connected,
    /// Connection lost, will retry.
    Reconnecting { attempt: u32 },
    /// Connection permanently failed.
    Failed { permanent: bool },
}

/// Tracks reconnection attempt timing and backoff.
#[derive(Component)]
pub struct ConnectionAttempt {
    pub next_retry: Instant,
    pub backoff: Duration,
}

impl ConnectionAttempt {
    pub const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
    pub const MAX_BACKOFF: Duration = Duration::from_secs(30);
    pub const BACKOFF_MULTIPLIER: u32 = 2;

    pub fn new() -> Self {
        Self {
            next_retry: Instant::now(),
            backoff: Self::INITIAL_BACKOFF,
        }
    }

    pub fn reset(&mut self) {
        self.backoff = Self::INITIAL_BACKOFF;
        self.next_retry = Instant::now();
    }

    pub fn increase_backoff(&mut self) {
        self.backoff = (self.backoff * Self::BACKOFF_MULTIPLIER).min(Self::MAX_BACKOFF);
        self.next_retry = Instant::now() + self.backoff;
    }

    pub fn is_ready(&self) -> bool {
        Instant::now() >= self.next_retry
    }
}

impl Default for ConnectionAttempt {
    fn default() -> Self {
        Self::new()
    }
}

