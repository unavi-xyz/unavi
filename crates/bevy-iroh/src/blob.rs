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
use tokio::sync::{
    oneshot,
    watch,
};
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
    rx:      async_channel::Receiver<Result<Bytes, BlobError>>,
    /// Dropping it cancels the fetch task.
    _cancel: oneshot::Sender<()>,
}

/// How a [`BlobRequest`] ended.
#[derive(Component)]
pub struct BlobResponse(pub Result<Bytes, BlobError>);

#[derive(Debug, Clone, Error)]
pub enum BlobError {
    #[error("blob too large: {size} bytes")]
    TooLarge { size: u64 },
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

/// The live provider list every fetch reads on each attempt.
#[derive(Resource)]
pub(crate) struct ProviderFeed(watch::Sender<Vec<EndpointId>>);

impl Default for ProviderFeed {
    fn default() -> Self {
        Self(watch::channel(Vec::new()).0)
    }
}

impl ProviderFeed {
    fn subscribe(&self) -> watch::Receiver<Vec<EndpointId>> {
        self.0.subscribe()
    }
}

/// Mirrors the sync targets and peers into the [`ProviderFeed`].
pub(crate) fn publish_providers(
    targets: Res<SyncTargets>,
    peers: Res<BlobProviders>,
    feed: Res<ProviderFeed>,
) {
    let list = provider_list(&targets, &peers);
    feed.0.send_if_modified(|current| {
        if *current == list {
            return false;
        }
        *current = list;
        true
    });
}

/// Requests with no fetch in flight and no answer, and requests just replaced.
type Unstarted = Or<(
    Changed<BlobRequest>,
    (Without<BlobPending>, Without<BlobResponse>),
)>;

/// Starts a fetch for every request that lacks one. Runs only once the store
/// exists; until then the requests stay unstarted and are picked up later.
pub(crate) fn start_blob_fetches(
    requests: Query<(Entity, &BlobRequest), Unstarted>,
    store: Res<DataStore>,
    feed: Res<ProviderFeed>,
    mut commands: Commands,
) {
    for (entity, request) in &requests {
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let (tx, rx) = async_channel::bounded(1);
        let hash = request.0;
        let store = store.0.clone();
        let providers = feed.subscribe();
        task::spawn(async move {
            fetch(hash, cancel_rx, tx, store, providers).await;
        });

        // Replaces a fetch already in flight, dropping its cancel sender.
        commands
            .entity(entity)
            .remove::<BlobResponse>()
            .insert(BlobPending {
                rx,
                _cancel: cancel_tx,
            });
    }
}

pub(crate) fn on_blob_request_remove(trigger: On<Remove, BlobRequest>, mut commands: Commands) {
    commands.entity(trigger.entity).try_remove::<BlobPending>();
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
    mut providers: watch::Receiver<Vec<EndpointId>>,
) {
    let mut cancel = std::pin::pin!(async move {
        cancel.await.ok();
    });

    // Retries until the request goes away, at the capped backoff, so a
    // provider that shows up late is still asked.
    let mut attempt = 0_u32;
    loop {
        let current = providers.borrow_and_update().clone();
        let res = tokio::select! {
            () = &mut cancel => return,
            () = providers_arrived(&mut providers, current.is_empty()) => continue,
            res = n0_future::time::timeout(
                ATTEMPT_TIMEOUT,
                get_blob(hash, &store, &current),
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
        tokio::select! {
            () = &mut cancel => return,
            () = n0_future::time::sleep(retry_delay(attempt)) => {}
        }
        attempt = attempt.saturating_add(1);
    }
}

/// Resolves once providers are known, when the attempt in progress started
/// with none. An attempt that had providers is left alone, so peer churn
/// cannot cancel a large transfer.
async fn providers_arrived(providers: &mut watch::Receiver<Vec<EndpointId>>, had_none: bool) {
    if had_none && providers.wait_for(|list| !list.is_empty()).await.is_ok() {
        return;
    }
    std::future::pending::<()>().await;
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

    /// How many requests the reconcile query selected, summed over frames.
    #[derive(Resource, Default)]
    struct Selected(usize);

    fn endpoint_id() -> EndpointId {
        SecretKey::generate().public()
    }

    fn spawn_pending(
        world: &mut World,
        rx: async_channel::Receiver<Result<Bytes, BlobError>>,
    ) -> Entity {
        world
            .spawn(BlobPending {
                rx,
                _cancel: oneshot::channel().0,
            })
            .id()
    }

    fn reconcile_app() -> App {
        let mut app = App::new();
        app.init_resource::<Selected>().add_systems(
            Update,
            |requests: Query<Entity, (With<BlobRequest>, Unstarted)>,
             mut selected: ResMut<Selected>| {
                selected.0 += requests.iter().count();
            },
        );
        app
    }

    /// Runs one frame and returns how many requests it selected.
    fn selected_in_frame(app: &mut App) -> usize {
        app.world_mut().resource_mut::<Selected>().0 = 0;
        app.update();
        app.world().resource::<Selected>().0
    }

    fn pending_marker() -> BlobPending {
        BlobPending {
            rx:      async_channel::bounded(1).1,
            _cancel: oneshot::channel().0,
        }
    }

    #[test]
    fn a_request_is_selected_every_frame_until_a_fetch_starts() {
        let mut app = reconcile_app();
        let entity = app.world_mut().spawn(BlobRequest(Hash::new(b"blob"))).id();

        assert_eq!(selected_in_frame(&mut app), 1);
        assert_eq!(
            selected_in_frame(&mut app),
            1,
            "a frame that could not start it leaves it for the next"
        );

        app.world_mut().entity_mut(entity).insert(pending_marker());
        assert_eq!(selected_in_frame(&mut app), 0, "a fetch in flight is left");
    }

    #[test]
    fn an_answered_request_is_left_alone_until_replaced() {
        let mut app = reconcile_app();
        let entity = app.world_mut().spawn(BlobRequest(Hash::new(b"blob"))).id();
        selected_in_frame(&mut app);
        app.world_mut()
            .entity_mut(entity)
            .insert(BlobResponse(Err(BlobError::TooLarge { size: 1 })));
        assert_eq!(selected_in_frame(&mut app), 0);

        app.world_mut()
            .entity_mut(entity)
            .insert(BlobRequest(Hash::new(b"other")));
        assert_eq!(
            selected_in_frame(&mut app),
            1,
            "a replaced request restarts"
        );
        assert_eq!(selected_in_frame(&mut app), 0, "and only once");
    }

    #[test]
    fn a_replaced_request_restarts_a_fetch_in_flight() {
        let mut app = reconcile_app();
        let entity = app
            .world_mut()
            .spawn((BlobRequest(Hash::new(b"blob")), pending_marker()))
            .id();
        selected_in_frame(&mut app);
        assert_eq!(selected_in_frame(&mut app), 0);

        app.world_mut()
            .entity_mut(entity)
            .insert(BlobRequest(Hash::new(b"other")));

        assert_eq!(selected_in_frame(&mut app), 1);
    }

    #[test]
    fn publishing_providers_updates_what_a_running_fetch_reads() {
        let (target, peer) = (endpoint_id(), endpoint_id());
        let mut app = App::new();
        app.init_resource::<ProviderFeed>()
            .insert_resource(SyncTargets(vec![iroh::EndpointAddr::new(target)]))
            .init_resource::<BlobProviders>()
            .add_systems(Update, publish_providers);
        let mut view = app.world().resource::<ProviderFeed>().subscribe();

        app.update();
        assert_eq!(*view.borrow_and_update(), vec![target]);

        app.world_mut().resource_mut::<BlobProviders>().0.push(peer);
        app.update();

        assert!(view.has_changed().expect("feed alive"));
        assert_eq!(
            *view.borrow_and_update(),
            vec![target, peer],
            "the same receiver sees providers that appeared after it subscribed"
        );
    }

    #[test]
    fn an_attempt_with_no_providers_wakes_once_one_is_published() {
        let feed = ProviderFeed::default();
        let mut view = feed.subscribe();
        feed.0.send_replace(vec![endpoint_id()]);

        bevy::tasks::block_on(providers_arrived(&mut view, true));
    }

    #[test]
    fn provider_list_puts_targets_first_and_dedupes() {
        let (a, b) = (endpoint_id(), endpoint_id());
        let targets = SyncTargets(vec![iroh::EndpointAddr::new(a)]);
        let peers = BlobProviders(vec![b, a]);

        assert_eq!(provider_list(&targets, &peers), vec![a, b]);
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
