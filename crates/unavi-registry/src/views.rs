use iroh_docs::{
    NamespaceId,
    api::Doc,
    protocol::Docs,
    store::Query,
};
use serde::{
    Deserialize,
    Serialize,
};
use wds::DataStore;

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

pub struct Views {
    ids: ViewIds,
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

/// Reopens the view named by `existing` and enrols it in this node's sync
/// set, so incoming requests for it are answered; a namespace outside the set
/// rejects reads with `NotFound`. A missing or unheld id mints a fresh view,
/// and the new ids are reported back to the operator for persistence.
async fn open_view(
    store: &DataStore,
    existing: Option<NamespaceId>,
) -> anyhow::Result<NamespaceId> {
    let held = match existing {
        Some(ns) => store.docs().api().open(ns).await?,
        None => None,
    };

    let doc = match held {
        Some(doc) => doc,
        None => store.docs().api().create().await?,
    };
    doc.start_sync(Vec::new()).await?;
    Ok(doc.id())
}

impl Views {
    pub async fn create(store: &DataStore, existing: Option<ViewIds>) -> anyhow::Result<Self> {
        let recent = open_view(store, existing.map(|v| v.recent)).await?;
        let featured = open_view(store, existing.map(|v| v.featured)).await?;
        let categories = open_view(store, existing.map(|v| v.categories)).await?;
        let active = open_view(store, existing.map(|v| v.active)).await?;

        Ok(Self {
            ids: ViewIds {
                recent,
                featured,
                categories,
                active,
            },
        })
    }

    pub async fn write_active(
        &self,
        docs: &Docs,
        active: &[ActiveSpace],
        capacity: usize,
    ) -> anyhow::Result<()> {
        let doc = self.open(docs, self.ids.active).await?;
        let author = docs.api().author_default().await?;

        doc.del(author, ACTIVE_PREFIX).await?;

        for (rank, space) in active.iter().take(capacity).enumerate() {
            let value = postcard::to_stdvec(&(space.occupants as u32, space.idle_secs))?;
            doc.set_bytes(author, active_key(rank, space.ns), value)
                .await?;
        }

        Ok(())
    }

    #[must_use]
    pub const fn ids(&self) -> ViewIds {
        self.ids
    }

    pub async fn rebuild(
        &self,
        docs: &Docs,
        catalog: &Catalog,
        blobs: &iroh_blobs::api::blobs::Blobs,
        config: &Config,
    ) -> anyhow::Result<()> {
        let mut live = catalog.live(docs, blobs).await?;

        live.sort_by_key(|s| std::cmp::Reverse(s.expires));
        let recent = live.iter().take(config.view_capacity).collect::<Vec<_>>();
        self.write(docs, self.ids.recent, &recent, ranked_key)
            .await?;

        let featured = live
            .iter()
            .filter(|s| config.featured.contains(&s.ns))
            .take(config.view_capacity)
            .collect::<Vec<_>>();
        self.write(docs, self.ids.featured, &featured, ranked_key)
            .await?;

        self.write_categories(docs, &live, config).await?;

        Ok(())
    }

    async fn open(&self, docs: &Docs, ns: NamespaceId) -> anyhow::Result<Doc> {
        docs.api()
            .open(ns)
            .await?
            .ok_or_else(|| anyhow::anyhow!("view doc {ns} not open"))
    }

    async fn write(
        &self,
        docs: &Docs,
        ns: NamespaceId,
        entries: &[&Submission],
        key: impl Fn(usize, NamespaceId) -> String,
    ) -> anyhow::Result<()> {
        let doc = self.open(docs, ns).await?;
        let author = docs.api().author_default().await?;

        doc.del(author, String::new()).await?;

        for (rank, submission) in entries.iter().enumerate() {
            let value = postcard::to_stdvec(submission)?;
            doc.set_bytes(author, key(rank, submission.ns), value)
                .await?;
        }

        Ok(())
    }

    async fn write_categories(
        &self,
        docs: &Docs,
        live: &[Submission],
        config: &Config,
    ) -> anyhow::Result<()> {
        let doc = self.open(docs, self.ids.categories).await?;
        let author = docs.api().author_default().await?;

        doc.del(author, String::new()).await?;

        for category in &config.categories {
            let matching = live
                .iter()
                .filter(|s| s.tags.iter().any(|t| t == category))
                .take(config.view_capacity);

            for (rank, submission) in matching.enumerate() {
                let value = postcard::to_stdvec(submission)?;
                doc.set_bytes(author, category_key(category, rank, submission.ns), value)
                    .await?;
            }
        }

        Ok(())
    }
}

/// Reads a view doc a client has synced, in the registry's intended order.
pub async fn read_view(
    docs: &Docs,
    blobs: &iroh_blobs::api::blobs::Blobs,
    ns: NamespaceId,
    prefix: &str,
) -> anyhow::Result<Vec<Submission>> {
    use futures::StreamExt;

    let Some(doc) = docs.api().open(ns).await? else {
        return Ok(Vec::new());
    };

    let query = Query::single_latest_per_key().key_prefix(prefix);
    let entries = doc.get_many(query).await?;
    let mut entries = std::pin::pin!(entries);

    let mut out = Vec::new();
    while let Some(entry) = entries.next().await {
        let Ok(bytes) = blobs.get_bytes(entry?.content_hash()).await else {
            continue;
        };
        if let Ok(submission) = postcard::from_bytes::<Submission>(&bytes) {
            out.push(submission);
        }
    }

    Ok(out)
}
