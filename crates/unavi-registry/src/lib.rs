use std::{
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            Ordering,
        },
    },
    time::Duration,
};

use iroh_blobs::api::blobs::Blobs;
use iroh_docs::protocol::Docs;
use tracing::warn;
use unavi_identity::auth::bindings::Bindings;
use unavi_store::store::Store;
use xdid::resolver::DidResolver;

use crate::{
    catalog::Catalog,
    config::Config,
    presence::PresenceTable,
    views::Views,
};

pub mod catalog;
pub mod client;
pub mod config;
pub mod control;
pub mod entry;
pub mod error;
pub mod follow;
pub mod presence;
pub mod views;

/// Bounds how long a newly occupied space stays invisible to clients.
const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(5);

pub struct RegistryContext {
    pub(crate) bindings: Arc<Bindings>,
    pub(crate) blobs:    Blobs,
    pub(crate) catalog:  Catalog,
    pub(crate) config:   Config,
    pub(crate) docs:     Docs,
    pub(crate) presence: PresenceTable,
    pub(crate) resolver: Arc<DidResolver>,
    pub(crate) views:    Views,
    dirty:               AtomicBool,
}

impl RegistryContext {
    pub(crate) fn request_rebuild(&self) {
        self.dirty.store(true, Ordering::Release);
    }
}

pub struct Registry {
    ctx: Arc<RegistryContext>,
}

impl Registry {
    /// Returns the registry and its iroh protocol handler, to be registered on
    /// the same router as the store's.
    pub async fn create(
        store: &Store,
        config: Config,
        bindings: Arc<Bindings>,
        resolver: Arc<DidResolver>,
    ) -> anyhow::Result<(Self, control::protocol::RegistryProtocol)> {
        let catalog = Catalog::create(store).await?;
        let views = Views::create(store).await?;

        let ctx = Arc::new(RegistryContext {
            bindings,
            blobs: store.blobs().clone(),
            catalog,
            config,
            dirty: AtomicBool::new(false),
            docs: store.docs().clone(),
            presence: PresenceTable::default(),
            resolver,
            views,
        });

        let protocol = control::protocol::RegistryProtocol::new(Arc::clone(&ctx));

        n0_future::task::spawn(maintenance(Arc::clone(&ctx)));

        Ok((Self { ctx }, protocol))
    }

    /// The namespaces clients sync to read this registry.
    #[must_use]
    pub fn views(&self) -> views::ViewIds {
        self.ctx.views.ids()
    }
}

async fn maintenance(ctx: Arc<RegistryContext>) {
    let window = ctx.config.activity_window;
    let mut published = Vec::new();

    loop {
        n0_future::time::sleep(MAINTENANCE_INTERVAL).await;

        ctx.presence.sweep(window).await;

        let active = ctx.presence.active(window).await;
        let ordering = active.iter().map(|s| s.ns).collect::<Vec<_>>();
        if ordering != published {
            match ctx
                .views
                .write_active(&ctx.docs, &active, ctx.config.view_capacity)
                .await
            {
                Ok(()) => published = ordering,
                Err(err) => warn!(?err, "active view write failed"),
            }
        }

        if ctx.dirty.swap(false, Ordering::AcqRel)
            && let Err(err) = ctx
                .views
                .rebuild(
                    &ctx.docs,
                    &ctx.catalog,
                    &ctx.blobs,
                    &ctx.config,
                    &ctx.resolver,
                )
                .await
        {
            warn!(?err, "view rebuild failed");
            ctx.request_rebuild();
        }
    }
}
