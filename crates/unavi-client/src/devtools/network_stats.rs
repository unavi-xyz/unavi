//! Network statistics tracking for debug monitoring.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use iroh::EndpointId;

use super::events::{NETWORK_EVENTS, NetworkEvent};

const BANDWIDTH_WINDOW_SIZE: usize = 60; // 1 second at 60 FPS.
const TICKRATE_WINDOW_SIZE: usize = 120; // 2 seconds at 60 FPS.

/// Smoothing factor for EMA. Lower = smoother but more lag.
const SMOOTHING_ALPHA: f32 = 0.15;

#[derive(Debug, Clone)]
pub struct PeerNetworkStats {
    // Bandwidth tracking.
    pub upload_bytes_per_sec: f32,
    pub download_bytes_per_sec: f32,
    pub iframe_upload_ratio: f32,
    pub pframe_upload_ratio: f32,

    // Tickrate tracking.
    pub effective_tickrate: f32,

    // Frame tracking.
    pub total_frames_received: u64,
    pub dropped_frames: u64,

    // Connection quality.
    pub quality_score: ConnectionQuality,
    pub estimated_latency_ms: f32,

    // Internal tracking.
    pub(crate) upload_samples: VecDeque<(Instant, usize)>,
    pub(crate) download_samples: VecDeque<(Instant, usize)>,
    pub(crate) iframe_bytes: usize,
    pub(crate) pframe_bytes: usize,
    pub(crate) tick_samples: VecDeque<Instant>,
    pub(crate) last_update: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl Default for PeerNetworkStats {
    fn default() -> Self {
        Self {
            upload_bytes_per_sec: 0.0,
            download_bytes_per_sec: 0.0,
            iframe_upload_ratio: 0.0,
            pframe_upload_ratio: 0.0,
            effective_tickrate: 0.0,
            total_frames_received: 0,
            dropped_frames: 0,
            quality_score: ConnectionQuality::Excellent,
            estimated_latency_ms: 0.0,
            upload_samples: VecDeque::with_capacity(BANDWIDTH_WINDOW_SIZE),
            download_samples: VecDeque::with_capacity(BANDWIDTH_WINDOW_SIZE),
            iframe_bytes: 0,
            pframe_bytes: 0,
            tick_samples: VecDeque::with_capacity(TICKRATE_WINDOW_SIZE),
            last_update: Instant::now(),
        }
    }
}

impl PeerNetworkStats {
    pub fn record_upload(&mut self, bytes: usize, is_iframe: bool) {
        let now = Instant::now();
        self.upload_samples.push_back((now, bytes));

        if is_iframe {
            self.iframe_bytes += bytes;
        } else {
            self.pframe_bytes += bytes;
        }

        // Keep only recent samples.
        while self.upload_samples.len() > BANDWIDTH_WINDOW_SIZE {
            self.upload_samples.pop_front();
        }
    }

    pub fn record_download(&mut self, bytes: usize) {
        let now = Instant::now();
        self.download_samples.push_back((now, bytes));

        // Keep only recent samples.
        while self.download_samples.len() > BANDWIDTH_WINDOW_SIZE {
            self.download_samples.pop_front();
        }
    }

    pub fn record_tick(&mut self) {
        let now = Instant::now();
        self.tick_samples.push_back(now);
        self.total_frames_received += 1;

        // Keep only recent samples.
        while self.tick_samples.len() > TICKRATE_WINDOW_SIZE {
            self.tick_samples.pop_front();
        }
    }

    pub const fn record_dropped_frame(&mut self) {
        self.dropped_frames += 1;
    }
}

#[derive(Resource, Default)]
pub struct NetworkStats {
    pub peers: HashMap<EndpointId, PeerNetworkStats>,
}

impl NetworkStats {
    pub fn get_or_create_peer(&mut self, peer: EndpointId) -> &mut PeerNetworkStats {
        self.peers.entry(peer).or_default()
    }
}

pub fn collect_network_events(mut stats: ResMut<NetworkStats>) {
    let mut guard = NETWORK_EVENTS.1.lock().expect("never poisons");

    while let Ok(event) = guard.try_recv() {
        match event {
            NetworkEvent::Download { peer, bytes } => {
                stats.get_or_create_peer(peer).record_download(bytes);
            }
            NetworkEvent::Upload {
                peer,
                bytes,
                is_iframe,
            } => {
                stats
                    .get_or_create_peer(peer)
                    .record_upload(bytes, is_iframe);
            }
            NetworkEvent::ValidTick { peer } => {
                stats.get_or_create_peer(peer).record_tick();
            }
            NetworkEvent::DroppedFrame { peer } => {
                stats.get_or_create_peer(peer).record_dropped_frame();
            }
        }
    }
}

pub fn update_bandwidth_stats(mut stats: ResMut<NetworkStats>) {
    let now = Instant::now();
    let window = Duration::from_secs(1);

    for peer_stats in stats.peers.values_mut() {
        // Calculate upload bandwidth.
        let cutoff = now.checked_sub(window).expect("value expected");
        peer_stats
            .upload_samples
            .retain(|(timestamp, _)| *timestamp > cutoff);

        let total_upload: usize = peer_stats
            .upload_samples
            .iter()
            .map(|(_, bytes)| bytes)
            .sum();
        let raw_upload = total_upload as f32;
        peer_stats.upload_bytes_per_sec =
            ema(peer_stats.upload_bytes_per_sec, raw_upload, SMOOTHING_ALPHA);

        // Calculate upload ratios.
        let total_tracked = peer_stats.iframe_bytes + peer_stats.pframe_bytes;
        if total_tracked > 0 {
            let iframe_ratio = peer_stats.iframe_bytes as f32 / total_tracked as f32;
            let pframe_ratio = peer_stats.pframe_bytes as f32 / total_tracked as f32;
            peer_stats.iframe_upload_ratio = ema(
                peer_stats.iframe_upload_ratio,
                iframe_ratio,
                SMOOTHING_ALPHA,
            );
            peer_stats.pframe_upload_ratio = ema(
                peer_stats.pframe_upload_ratio,
                pframe_ratio,
                SMOOTHING_ALPHA,
            );
        }

        // Calculate download bandwidth.
        peer_stats
            .download_samples
            .retain(|(timestamp, _)| *timestamp > cutoff);

        let total_download: usize = peer_stats
            .download_samples
            .iter()
            .map(|(_, bytes)| bytes)
            .sum();
        let raw_download = total_download as f32;
        peer_stats.download_bytes_per_sec = ema(
            peer_stats.download_bytes_per_sec,
            raw_download,
            SMOOTHING_ALPHA,
        );

        peer_stats.last_update = now;
    }
}

pub fn update_tickrate_stats(mut stats: ResMut<NetworkStats>) {
    let now = Instant::now();
    let window = Duration::from_secs(1);

    for peer_stats in stats.peers.values_mut() {
        let cutoff = now.checked_sub(window).expect("value expected");
        peer_stats
            .tick_samples
            .retain(|timestamp| *timestamp > cutoff);

        let tick_count = peer_stats.tick_samples.len();
        let raw_tickrate = tick_count as f32;
        peer_stats.effective_tickrate =
            ema(peer_stats.effective_tickrate, raw_tickrate, SMOOTHING_ALPHA);

        // Estimate latency from tickrate variance (simple heuristic).
        if tick_count > 2 {
            let mut intervals = Vec::new();
            for i in 1..peer_stats.tick_samples.len() {
                let interval = peer_stats.tick_samples[i]
                    .duration_since(peer_stats.tick_samples[i - 1])
                    .as_millis() as f32;
                intervals.push(interval);
            }

            if !intervals.is_empty() {
                let avg_interval = intervals.iter().sum::<f32>() / intervals.len() as f32;
                let variance: f32 = intervals
                    .iter()
                    .map(|i| (i - avg_interval).powi(2))
                    .sum::<f32>()
                    / intervals.len() as f32;
                let raw_latency = avg_interval + variance.sqrt();
                peer_stats.estimated_latency_ms = ema(
                    peer_stats.estimated_latency_ms,
                    raw_latency,
                    SMOOTHING_ALPHA,
                );
            }
        }
    }
}

pub fn detect_dropped_frames(mut stats: ResMut<NetworkStats>) {
    for peer_stats in stats.peers.values_mut() {
        // Calculate connection quality based on dropped frames and tickrate.
        let drop_rate = if peer_stats.total_frames_received > 0 {
            peer_stats.dropped_frames as f32 / peer_stats.total_frames_received as f32
        } else {
            0.0
        };

        peer_stats.quality_score = if drop_rate < 0.01 && peer_stats.effective_tickrate >= 19.0 {
            ConnectionQuality::Excellent
        } else if drop_rate < 0.05 && peer_stats.effective_tickrate > 15.0 {
            ConnectionQuality::Good
        } else if drop_rate < 0.15 {
            ConnectionQuality::Fair
        } else {
            ConnectionQuality::Poor
        };
    }
}

/// Exponential moving average for smoothing display values.
fn ema(current: f32, new_value: f32, alpha: f32) -> f32 {
    alpha.mul_add(new_value, (1.0 - alpha) * current)
}
