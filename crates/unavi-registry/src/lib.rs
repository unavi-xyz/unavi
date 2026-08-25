//! Discovery and curation, separate from [`wds`].

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

/// Bounds how long a newly occupied space stays invisible to clients.
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
    /// Returns the registry and its iroh protocol handler, to be registered on
    /// the same router as the store's.
    pub async fn create(
        store: Arc<DataStore>,
        config: Config,
    ) -> anyhow::Result<(Self, IrohProtocol<control::RegistryService>)> {
        let docs = store.docs().clone();
        let blobs = store.blobs().blobs().clone();

        let catalog = Catalog::create(&docs, &store).await?;
        let views = Views::create(&docs, &store).await?;

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
