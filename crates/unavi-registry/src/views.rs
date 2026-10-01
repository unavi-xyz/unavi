use iroh_docs::NamespaceId;
use serde::{
    Deserialize,
    Serialize,
};
use unavi_store::{
    Document,
    Store,
};
use xdid::resolver::DidResolver;

use crate::{
    catalog::Catalog,
    config::Config,
    entry::Submission,
    presence::ActiveSpace,
};

/// Prefix of the active-spaces view; clients filter for activity by it.
pub const ACTIVE_PREFIX: &str = "active/";

/// Namespaces of the docs a registry publishes for clients to sync.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ViewIds {
    pub recent:     NamespaceId,
    pub featured:   NamespaceId,
    pub categories: NamespaceId,
    pub active:     NamespaceId,
}

/// The documents behind [`ViewIds`], held open for as long as this registry
/// runs. Releasing one takes it back out of the sync set that answers clients.
pub struct Views {
    recent:     Document,
    featured:   Document,
    categories: Document,
    active:     Document,
}

/// Zero-padded so key order is the registry's intended order; clients need no
/// sorting pass.
fn ranked_key(rank: usize, ns: NamespaceId) -> String {
    format!("{rank:08}/{ns}")
}

fn category_key(tag: &str, rank: usize, ns: NamespaceId) -> String {
    format!("{tag}/{rank:08}/{ns}")
}

fn active_key(rank: usize, ns: NamespaceId) -> String {
    format!("{ACTIVE_PREFIX}{rank:08}/{ns}")
}

/// Each view is recorded under its own key, so one lost view is reminted
/// without disturbing the others.
async fn open_view(store: &Store, name: &str) -> anyhow::Result<Document> {
    let view = store.named(&format!("registry/views/{name}")).await?;
    view.serve().await?;
    Ok(view)
}

impl Views {
    /// Reopens each view this node recorded, minting any that is absent or
    /// whose capability is no longer held, and enrols every one in the sync
    /// set: a namespace outside that set rejects reads with `NotFound`.
    pub async fn create(store: &Store) -> anyhow::Result<Self> {
        Ok(Self {
            recent:     open_view(store, "recent").await?,
            featured:   open_view(store, "featured").await?,
            categories: open_view(store, "categories").await?,
            active:     open_view(store, "active").await?,
        })
    }

    pub async fn write_active(
        &self,
        active: &[ActiveSpace],
        capacity: usize,
    ) -> anyhow::Result<()> {
        self.active.remove_prefix(ACTIVE_PREFIX).await?;

        for (rank, space) in active.iter().take(capacity).enumerate() {
            let value = postcard::to_stdvec(&(space.occupants as u32, space.idle_secs))?;
            self.active.set(active_key(rank, space.ns), value).await?;
        }

        Ok(())
    }

    #[must_use]
    pub fn ids(&self) -> ViewIds {
        ViewIds {
            recent:     self.recent.id(),
            featured:   self.featured.id(),
            categories: self.categories.id(),
            active:     self.active.id(),
        }
    }

    pub async fn rebuild(
        &self,
        catalog: &Catalog,
        config: &Config,
        resolver: &DidResolver,
    ) -> anyhow::Result<()> {
        let mut live = catalog.live(resolver).await?;

        live.sort_by_key(|s| std::cmp::Reverse(s.expires));
        let recent = live.iter().take(config.view_capacity).collect::<Vec<_>>();
        write(&self.recent, &recent, ranked_key).await?;

        let featured = live
            .iter()
            .filter(|s| config.featured.contains(&s.ns))
            .take(config.view_capacity)
            .collect::<Vec<_>>();
        write(&self.featured, &featured, ranked_key).await?;

        self.write_categories(&live, config).await?;

        Ok(())
    }

    async fn write_categories(&self, live: &[Submission], config: &Config) -> anyhow::Result<()> {
        self.categories.remove_prefix("").await?;

        for category in &config.categories {
            let matching = live
                .iter()
                .filter(|s| s.tags.iter().any(|t| t == category))
                .take(config.view_capacity);

            for (rank, submission) in matching.enumerate() {
                let value = postcard::to_stdvec(submission)?;
                self.categories
                    .set(category_key(category, rank, submission.ns), value)
                    .await?;
            }
        }

        Ok(())
    }
}

async fn write(
    view: &Document,
    entries: &[&Submission],
    key: impl Fn(usize, NamespaceId) -> String,
) -> anyhow::Result<()> {
    view.remove_prefix("").await?;

    for (rank, submission) in entries.iter().enumerate() {
        let value = postcard::to_stdvec(submission)?;
        view.set(key(rank, submission.ns), value).await?;
    }

    Ok(())
}

/// Reads a view doc a client has synced, in the registry's intended order.
pub async fn read_view(
    store: &Store,
    ns: NamespaceId,
    prefix: &str,
) -> anyhow::Result<Vec<Submission>> {
    let Some(doc) = store.held(ns).await? else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    for entry in doc.list(prefix).await? {
        let Some(bytes) = doc.value(&entry).await? else {
            continue;
        };
        if let Ok(submission) = postcard::from_bytes::<Submission>(&bytes) {
            out.push(submission);
        }
    }

    Ok(out)
}
