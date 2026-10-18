//! Fetching one blob by hash into the ECS: insert a [`BlobRequest`], read the
//! [`BlobResponse`] that lands beside it.
//!
//! A fetch asks the sync targets, then the [`BlobProviders`], retrying with
//! backoff, and reads at most [`MAX_BLOB_BYTES`].

use std::{
    sync::Arc,
    time::Duration,
};

use async_channel::Sender;
use bevy::{
    prelude::*,
    tasks::futures_lite::StreamExt,
};
use bevy_async::task;
use bytes::Bytes;
use iroh::EndpointId;
use iroh_blobs::{
    Hash,
    HashAndFormat,
    api::blobs::Blobs,
};
use n0_future::time::Instant;
use thiserror::Error;
use tokio::sync::oneshot;
use unavi_store::Store;

use crate::store::{
    BlobProviders,
    DataStore,
    SyncTargets,
};

/// Every read lands the whole blob in memory at once, so this bounds a single
/// allocation as well as a transfer. A blob is the content of one entry, so it
/// shares the store's entry cap.
pub const MAX_BLOB_BYTES: u64 = unavi_store::MAX_ENTRY_BYTES;
/// Cap on one download attempt, so a wedged sync cannot stall a fetch forever.
const ATTEMPT_TIMEOUT: Duration = Duration::from_mins(5);
/// First retry waits this long; each attempt doubles until `MAX_RETRY_DELAY`.
const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(1);
const MAX_RETRY_DELAY: Duration = Duration::from_mins(1);
const MAX_ATTEMPTS: u32 = 10;
/// Progress is logged at most once per step and once per interval, so a large
/// transfer emits a handful of lines and a quick one emits none. The lines are
/// there to show a stalled or slow download, not to narrate every fetch.
const PROGRESS_STEP: f64 = 0.1;
const PROGRESS_INTERVAL: Duration = Duration::from_secs(5);

/// Fetches the blob `hash`. Replacing it cancels the fetch in flight and starts
/// another; removing it cancels.
#[derive(Component)]
pub struct BlobRequest(pub Hash);

/// A fetch in flight for this entity's [`BlobRequest`].
#[derive(Component)]
pub struct BlobPending {
    rx:        async_channel::Receiver<Result<Bytes, BlobError>>,
    /// The providers the fetch started with.
    providers: Vec<EndpointId>,
    /// Dropping it cancels the fetch task.
    _cancel:   oneshot::Sender<()>,
}

/// How a [`BlobRequest`] ended.
#[derive(Component)]
pub struct BlobResponse(pub Result<Bytes, BlobError>);

#[derive(Debug, Clone, Error)]
pub enum BlobError {
    #[error("blob too large: {size} bytes")]
    TooLarge { size: u64 },
    #[error("fetch failed after {attempts} attempts")]
    Exhausted { attempts: u32 },
    #[error("fetch failed: {0:#}")]
    Io(Arc<anyhow::Error>),
}

impl BlobError {
    /// Permanent failures abort immediately; only network-style failures retry.
    pub(crate) const fn retryable(&self) -> bool {
        matches!(self, Self::Io(_))
    }

    fn io(err: impl Into<anyhow::Error>) -> Self {
        Self::Io(Arc::new(err.into()))
    }
}

fn progress_step(progress: f64) -> u32 {
    (progress / PROGRESS_STEP) as u32
}

/// Decides which progress readings of one download are worth a log line.
struct ProgressLog {
    step: u32,
    at:   Instant,
}

impl ProgressLog {
    /// The interval runs from `start`, so a download that completes within it
    /// never logs.
    const fn new(start: Instant) -> Self {
        Self {
            step: 0,
            at:   start,
        }
    }

    fn should_log(&mut self, progress: f64, now: Instant) -> bool {
        let step = progress_step(progress);
        if step <= self.step || now.duration_since(self.at) < PROGRESS_INTERVAL {
            return false;
        }
        self.step = step;
        self.at = now;
        true
    }
}

/// The backoff before attempt `attempt` (0-indexed), doubling to a cap.
fn retry_delay(attempt: u32) -> Duration {
    Duration::from_secs(
        INITIAL_RETRY_DELAY
            .as_secs()
            .checked_shl(attempt)
            .unwrap_or(u64::MAX)
            .min(MAX_RETRY_DELAY.as_secs()),
    )
}

/// Sync targets first, so a configured node is still tried before the peers.
fn provider_list(targets: &SyncTargets, peers: &BlobProviders) -> Vec<EndpointId> {
    let mut providers = targets.0.iter().map(|addr| addr.id).collect::<Vec<_>>();
    for id in &peers.0 {
        if !providers.contains(id) {
            providers.push(*id);
        }
    }
    providers
}

/// Whether a fetch started with `snapshot` never asked a sync target, or
/// asked nobody while providers are now known. A peer joining otherwise leaves
/// a running fetch alone, so peer churn cannot cancel large transfers.
fn needs_restart(snapshot: &[EndpointId], targets: &[EndpointId], current: &[EndpointId]) -> bool {
    targets.iter().any(|id| !snapshot.contains(id)) || (snapshot.is_empty() && !current.is_empty())
}

pub(crate) fn on_blob_request_insert(
    trigger: On<Insert, BlobRequest>,
    requests: Query<&BlobRequest>,
    store: Option<Res<DataStore>>,
    targets: Res<SyncTargets>,
    peers: Res<BlobProviders>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    let Ok(request) = requests.get(entity) else {
        return;
    };
    let Some(store) = store else {
        warn!(%entity, "cannot fetch a blob before the store exists");
        return;
    };

    let providers = provider_list(&targets, &peers);
    let (cancel_tx, cancel_rx) = oneshot::channel();
    let (tx, rx) = async_channel::bounded(1);
    let hash = request.0;
    let store = store.0.clone();
    let snapshot = providers.clone();
    task::spawn(async move {
        fetch(hash, cancel_rx, tx, store, providers).await;
    });

    // Replaces a fetch already in flight, dropping its cancel sender.
    commands
        .entity(entity)
        .remove::<BlobResponse>()
        .insert(BlobPending {
            rx,
            providers: snapshot,
            _cancel: cancel_tx,
        });
}

pub(crate) fn on_blob_request_remove(trigger: On<Remove, BlobRequest>, mut commands: Commands) {
    commands.entity(trigger.entity).try_remove::<BlobPending>();
}

/// Restarts fetches that began before a provider they could use was known.
pub(crate) fn restart_stale_fetches(
    targets: Res<SyncTargets>,
    peers: Res<BlobProviders>,
    pending: Query<(Entity, &BlobRequest, &BlobPending)>,
    mut commands: Commands,
) {
    let current = provider_list(&targets, &peers);
    let target_ids = targets.0.iter().map(|addr| addr.id).collect::<Vec<_>>();
    for (entity, request, load) in &pending {
        if needs_restart(&load.providers, &target_ids, &current) {
            commands.entity(entity).insert(BlobRequest(request.0));
        }
    }
}

pub(crate) fn recv_blob_responses(mut commands: Commands, loading: Query<(Entity, &BlobPending)>) {
    for (entity, load) in loading {
        let response = match load.rx.try_recv() {
            Ok(response) => response,
            Err(async_channel::TryRecvError::Empty) => continue,
            Err(async_channel::TryRecvError::Closed) => {
                warn!(?entity, "blob fetch ended without a response");
                Err(BlobError::io(anyhow::anyhow!(
                    "fetch task ended without a response"
                )))
            }
        };
        commands
            .entity(entity)
            .try_remove::<BlobPending>()
            .try_insert(BlobResponse(response));
    }
}

async fn fetch(
    hash: Hash,
    cancel: oneshot::Receiver<()>,
    tx: Sender<Result<Bytes, BlobError>>,
    store: Store,
    providers: Vec<EndpointId>,
) {
    let mut cancel = std::pin::pin!(async move {
        cancel.await.ok();
    });

    for attempt in 0..MAX_ATTEMPTS {
        let res = tokio::select! {
            () = &mut cancel => return,
            res = n0_future::time::timeout(
                ATTEMPT_TIMEOUT,
                get_blob(hash, &store, &providers),
            ) => res,
        };
        match res {
            Ok(Ok(bytes)) => {
                tx.send(Ok(bytes)).await.ok();
                return;
            }
            Ok(Err(err)) if !err.retryable() => {
                tx.send(Err(err)).await.ok();
                return;
            }
            Ok(Err(err)) => warn!(%hash, attempt, ?err, "blob fetch failed, retrying"),
            Err(_) => warn!(%hash, attempt, "blob fetch attempt timed out"),
        }
        n0_future::time::sleep(retry_delay(attempt)).await;
    }

    tx.send(Err(BlobError::Exhausted {
        attempts: MAX_ATTEMPTS,
    }))
    .await
    .ok();
}

/// Bounds the read of a fetched blob and of an already cached copy, which may
/// have been written by a path that did not bound it.
async fn get_blob(hash: Hash, store: &Store, providers: &[EndpointId]) -> Result<Bytes, BlobError> {
    let blobs = store.blobs();

    // No document references a hash that arrived as a bare id, so a sweep is
    // free to reclaim it, partial blob included, while the download is still
    // writing. The tag is taken before the fetch and released once the bytes
    // are in hand.
    let batch = blobs.batch().await.map_err(BlobError::io)?;
    let _guard = batch
        .temp_tag(HashAndFormat::raw(hash))
        .await
        .map_err(BlobError::io)?;

    if blobs.has(hash).await.map_err(BlobError::io)? {
        return read_bounded(hash, blobs).await;
    }

    if providers.is_empty() {
        // Content a joined document fetches arrives without a provider list.
        watch_until_complete(hash, blobs).await?;
    } else {
        // Watching alongside the download keeps the size bound enforced
        // mid-transfer, and cancels a download that outgrows it.
        tokio::select! {
            res = store.downloader().download(hash, providers.to_vec()) => res.map_err(BlobError::io)?,
            res = watch_until_complete(hash, blobs) => res?,
        }
    }

    read_bounded(hash, blobs).await
}

async fn watch_until_complete(hash: Hash, blobs: &Blobs) -> Result<(), BlobError> {
    let mut stream = blobs.observe(hash).stream().await.map_err(BlobError::io)?;

    let mut log = ProgressLog::new(Instant::now());

    while let Some(field) = stream.next().await {
        let size = field.size();

        if size > MAX_BLOB_BYTES {
            return Err(BlobError::TooLarge { size });
        }

        if field.is_complete() {
            return Ok(());
        }

        if size > 0 {
            // `validated_size` only resolves once the final chunk lands, so
            // received bytes are what a transfer's progress reads from.
            let progress = field.total_bytes() as f64 / size as f64;
            if log.should_log(progress, Instant::now()) {
                info!(hash = %hash, "Downloading: {:.0}%", progress * 100.0);
            }
        }
    }

    Ok(())
}

async fn read_bounded(hash: Hash, blobs: &Blobs) -> Result<Bytes, BlobError> {
    let size = blobs.observe(hash).await.map_err(BlobError::io)?.size();
    if size > MAX_BLOB_BYTES {
        return Err(BlobError::TooLarge { size });
    }
    blobs.get_bytes(hash).await.map_err(BlobError::io)
}

#[cfg(test)]
mod tests {
    use iroh::SecretKey;

    use super::*;

    #[derive(Resource, Default)]
    struct Restarts(u32);

    fn endpoint_id() -> EndpointId {
        SecretKey::generate().public()
    }

    fn spawn_pending_with(
        world: &mut World,
        rx: async_channel::Receiver<Result<Bytes, BlobError>>,
        providers: Vec<EndpointId>,
    ) -> Entity {
        world
            .spawn(BlobPending {
                rx,
                providers,
                _cancel: oneshot::channel().0,
            })
            .id()
    }

    fn spawn_pending(
        world: &mut World,
        rx: async_channel::Receiver<Result<Bytes, BlobError>>,
    ) -> Entity {
        spawn_pending_with(world, rx, Vec::new())
    }

    fn restart_app(known: EndpointId) -> App {
        let mut app = App::new();
        app.init_resource::<SyncTargets>()
            .init_resource::<Restarts>()
            .insert_resource(BlobProviders(vec![known]))
            .add_observer(
                |_: On<Insert, BlobRequest>, mut restarts: ResMut<Restarts>| {
                    restarts.0 += 1;
                },
            )
            .add_systems(Update, restart_stale_fetches);
        app
    }

    fn spawn_request(app: &mut App, snapshot: Vec<EndpointId>) {
        let (_tx, rx) = async_channel::bounded(1);
        let entity = spawn_pending_with(app.world_mut(), rx, snapshot);
        app.world_mut()
            .entity_mut(entity)
            .insert(BlobRequest(Hash::new(b"blob")));
        app.world_mut().resource_mut::<Restarts>().0 = 0;
    }

    #[test]
    fn provider_list_puts_targets_first_and_dedupes() {
        let (a, b) = (endpoint_id(), endpoint_id());
        let targets = SyncTargets(vec![iroh::EndpointAddr::new(a)]);
        let peers = BlobProviders(vec![b, a]);

        assert_eq!(provider_list(&targets, &peers), vec![a, b]);
    }

    #[test]
    fn a_fetch_restarts_for_a_missed_target_or_from_nobody_but_not_for_a_new_peer() {
        let (target, peer, other) = (endpoint_id(), endpoint_id(), endpoint_id());

        assert!(
            needs_restart(&[], &[], &[peer]),
            "asked nobody, a peer is known"
        );
        assert!(
            needs_restart(&[peer], &[target], &[target, peer]),
            "missed a target"
        );
        assert!(
            !needs_restart(&[target, peer], &[target], &[target, peer, other]),
            "a new peer leaves a running fetch alone"
        );
        assert!(!needs_restart(&[], &[], &[]));
    }

    #[test]
    fn a_pending_fetch_with_no_providers_restarts_once_one_is_known() {
        let mut app = restart_app(endpoint_id());
        spawn_request(&mut app, Vec::new());

        app.update();

        assert_eq!(app.world().resource::<Restarts>().0, 1);
    }

    #[test]
    fn a_pending_fetch_that_asked_every_provider_is_left_running() {
        let known = endpoint_id();
        let mut app = restart_app(known);
        spawn_request(&mut app, vec![known]);

        app.update();

        assert_eq!(app.world().resource::<Restarts>().0, 0);
    }

    #[test]
    fn retry_delay_doubles_then_caps() {
        assert_eq!(retry_delay(0), Duration::from_secs(1));
        assert_eq!(retry_delay(1), Duration::from_secs(2));
        assert_eq!(retry_delay(2), Duration::from_secs(4));
        assert_eq!(retry_delay(30), MAX_RETRY_DELAY);
    }

    #[test]
    fn progress_only_steps_forward_in_tenths() {
        assert_eq!(progress_step(0.0), 0);
        assert_eq!(progress_step(0.0999), 0, "a chunk-sized delta logs nothing");
        assert_eq!(progress_step(0.1), 1);
        assert_eq!(progress_step(0.55), 5);
        assert_eq!(progress_step(1.0), 10);
    }

    #[test]
    fn progress_logs_wait_for_both_a_step_and_the_interval() {
        let start = Instant::now();
        let mut log = ProgressLog::new(start);

        assert!(
            !log.should_log(0.9, start + Duration::from_secs(1)),
            "a download inside its first interval logs nothing"
        );
        assert!(log.should_log(0.2, start + PROGRESS_INTERVAL));
        assert!(
            !log.should_log(0.25, start + PROGRESS_INTERVAL * 3),
            "no new step, no line"
        );
        assert!(
            !log.should_log(0.5, start + PROGRESS_INTERVAL + Duration::from_secs(1)),
            "a new step inside the interval waits"
        );
        assert!(log.should_log(0.5, start + PROGRESS_INTERVAL * 2));
    }

    #[test]
    fn only_io_errors_are_retryable() {
        assert!(BlobError::io(anyhow::anyhow!("network blip")).retryable());
        assert!(!BlobError::TooLarge { size: 1 }.retryable());
        assert!(!BlobError::Exhausted { attempts: 3 }.retryable());
    }

    #[test]
    fn a_failed_fetch_surfaces_an_error_response() {
        let mut app = App::new();
        app.add_systems(Update, recv_blob_responses);
        let (tx, rx) = async_channel::bounded(1);
        let entity = spawn_pending(app.world_mut(), rx);
        tx.send_blocking(Err(BlobError::TooLarge { size: 1 }))
            .expect("send");

        app.update();

        let world = app.world_mut();
        let response = world.get::<BlobResponse>(entity).expect("response");
        assert!(matches!(response.0, Err(BlobError::TooLarge { .. })));
        assert!(
            world.get::<BlobPending>(entity).is_none(),
            "a resolved fetch drops its pending state"
        );
    }

    #[test]
    fn a_dropped_channel_reports_failure() {
        let mut app = App::new();
        app.add_systems(Update, recv_blob_responses);
        let (tx, rx) = async_channel::bounded(1);
        let entity = spawn_pending(app.world_mut(), rx);
        drop(tx);

        app.update();

        let world = app.world_mut();
        let response = world.get::<BlobResponse>(entity).expect("response");
        assert!(
            matches!(response.0, Err(BlobError::Io(_))),
            "a task that vanished mid-fetch is a failure, not a hang"
        );
    }
}
