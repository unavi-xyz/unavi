//! Discovery and curation, deliberately separate from [`wds`].
//!
//! WDS is agnostic by design: a host stores bytes it never interprets. A
//! registry is the opposite kind of service — ranking, categorising and
//! searching all require reading what was submitted. The two cannot share a
//! crate without one losing its defining property, so they share a node
//! instead: a registry reuses the store's endpoint, docs, blobs and
//! authenticated sessions while keeping its own protocol and its own opinions.

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
use irpc::Client;
use irpc_iroh::IrohProtocol;
use tracing::warn;
use wds::DataStore;

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
pub mod presence;
pub mod views;

/// How often views are rebuilt when submissions have changed, and expired
/// presence is swept.
///
/// Bounds how long a newly occupied space stays invisible to clients, so it is
/// kept short; the work is skipped entirely when nothing changed.
const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(5);

pub struct RegistryContext {
    pub(crate) blobs:    Blobs,
    pub(crate) catalog:  Catalog,
    pub(crate) config:   Config,
    pub(crate) docs:     Docs,
    pub(crate) presence: PresenceTable,
    pub(crate) store:    Arc<DataStore>,
    pub(crate) views:    Views,
    dirty:               AtomicBool,
}

impl RegistryContext {
    /// Marks views stale. The maintenance loop coalesces bursts of submissions
    /// into one rebuild rather than rewriting every view per write.
    pub(crate) fn request_rebuild(&self) {
        self.dirty.store(true, Ordering::Release);
    }

    pub(crate) async fn session_did(
        &self,
        token: &wds::SessionToken,
    ) -> Option<xdid::core::did::Did> {
        self.store.session_did(token).await
    }
}

pub struct Registry {
    client: Client<control::RegistryService>,
    ctx:    Arc<RegistryContext>,
}

impl Registry {
    /// Builds a registry sharing `store`'s endpoint, docs, blobs and sessions.
    ///
    /// Returns the registry and its iroh protocol handler, to be registered on
    /// the same router as the store's.
    pub async fn create(
        store: Arc<DataStore>,
        config: Config,
    ) -> anyhow::Result<(Self, IrohProtocol<control::RegistryService>)> {
        let docs = store.docs().clone();
        let blobs = store.blobs().blobs().clone();

        let catalog = Catalog::create(&docs).await?;
        let views = Views::create(&docs).await?;

        let ctx = Arc::new(RegistryContext {
            blobs,
            catalog,
            config,
            dirty: AtomicBool::new(false),
            docs,
            presence: PresenceTable::default(),
            store,
            views,
        });

        let (client, protocol) = control::protocol(Arc::clone(&ctx));

        n0_future::task::spawn(maintenance(Arc::clone(&ctx)));

        Ok((Self { client, ctx }, protocol))
    }

    /// The namespaces clients sync to read this registry.
    #[must_use]
    pub fn views(&self) -> views::ViewIds {
        self.ctx.views.ids()
    }

    #[must_use]
    pub const fn client(&self) -> &Client<control::RegistryService> {
        &self.client
    }
}

async fn maintenance(ctx: Arc<RegistryContext>) {
    let window = ctx.config.activity_window;
    let mut published = Vec::new();

    loop {
        n0_future::time::sleep(MAINTENANCE_INTERVAL).await;

        ctx.presence.sweep(window).await;

        // Rewritten only when the ordered set of active spaces changes, so a
        // steady room costs no doc writes however often its peers heartbeat.
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
                .rebuild(&ctx.docs, &ctx.catalog, &ctx.blobs, &ctx.config)
                .await
        {
            warn!(?err, "view rebuild failed");
            ctx.request_rebuild();
        }
    }
}
