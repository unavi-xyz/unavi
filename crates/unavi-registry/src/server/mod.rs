//! Running a registry.

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

use n0_future::task::AbortOnDropHandle;
use tracing::warn;
use unavi_identity::{
    auth::Bindings,
    resolver::Resolver,
};
use unavi_store::Store;

pub use crate::server::{
    config::{
        Config,
        Submitters,
    },
    protocol::RegistryProtocol,
};
use crate::{
    server::{
        catalog::Catalog,
        presence::PresenceTable,
        views::Views,
    },
    views::ViewIds,
};

mod catalog;
mod config;
mod handlers;
mod presence;
mod protocol;
mod views;

/// Bounds how long a newly occupied space stays invisible to clients.
const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(5);

/// How often expired listings are deleted from the catalog.
const PRUNE_EVERY: u32 = 120;

pub(crate) struct Shared {
    bindings: Arc<Bindings>,
    catalog:  Catalog,
    config:   Config,
    presence: PresenceTable,
    resolver: Arc<Resolver>,
    views:    Views,
    dirty:    AtomicBool,
}

impl Shared {
    fn request_rebuild(&self) {
        self.dirty.store(true, Ordering::Release);
    }
}

/// A running registry. Maintenance stops when this drops.
pub struct Registry {
    shared:       Arc<Shared>,
    _maintenance: AbortOnDropHandle<()>,
}

impl Registry {
    /// Returns the registry and its protocol handler, to be registered under
    /// [`crate::rpc::ALPN`] on the same router as the store's.
    ///
    /// `bindings` must be the ones `wired/auth` fills on that router's
    /// endpoint, since callers are identified through them.
    pub async fn create(
        store: &Store,
        config: Config,
        bindings: Arc<Bindings>,
        resolver: Arc<Resolver>,
    ) -> anyhow::Result<(Self, RegistryProtocol)> {
        let catalog = Catalog::create(store).await?;
        let views = Views::create(store).await?;

        let shared = Arc::new(Shared {
            bindings,
            catalog,
            config,
            // Rebuilt once at start, so views catch up with the catalog.
            dirty: AtomicBool::new(true),
            presence: PresenceTable::default(),
            resolver,
            views,
        });

        let protocol = RegistryProtocol::new(Arc::clone(&shared));
        let maintenance = n0_future::task::spawn(maintenance(Arc::clone(&shared)));

        Ok((
            Self {
                shared,
                _maintenance: AbortOnDropHandle::new(maintenance),
            },
            protocol,
        ))
    }

    /// The namespaces clients sync to read this registry.
    #[must_use]
    pub fn views(&self) -> ViewIds {
        self.shared.views.ids()
    }
}

async fn maintenance(shared: Arc<Shared>) {
    let window = shared.config.activity_window;
    let mut tick = 0_u32;

    loop {
        n0_future::time::sleep(MAINTENANCE_INTERVAL).await;
        tick = tick.wrapping_add(1);

        shared.presence.sweep(window);
        let active = shared.presence.active(window);
        if let Err(err) = shared
            .views
            .write_active(&active, shared.config.view_capacity)
            .await
        {
            warn!(?err, "active view write failed");
        }

        if tick.is_multiple_of(PRUNE_EVERY) {
            match shared.catalog.prune().await {
                Ok(0) => {}
                Ok(_) => shared.request_rebuild(),
                Err(err) => warn!(?err, "catalog prune failed"),
            }
        }

        if shared.dirty.swap(false, Ordering::AcqRel)
            && let Err(err) = shared
                .views
                .rebuild(shared.catalog.live(), &shared.config)
                .await
        {
            warn!(?err, "view rebuild failed");
            shared.request_rebuild();
        }
    }
}
