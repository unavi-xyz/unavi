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

use crate::{
    catalog::Catalog,
    config::Config,
    entry::Submission,
    presence::ActiveSpace,
};

/// Prefix of the active-spaces view, so a client listing every view namespace
/// picks out activity entries without needing to know which doc is which.
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

/// Rank is zero-padded into the key so a doc's natural key order is the
/// registry's intended order, and a client needs no sorting pass.
fn ranked_key(rank: usize, ns: NamespaceId) -> String {
    format!("{rank:08}/{ns}")
}

fn category_key(tag: &str, rank: usize, ns: NamespaceId) -> String {
    format!("{tag}/{rank:08}/{ns}")
}

fn active_key(rank: usize, ns: NamespaceId) -> String {
    format!("{ACTIVE_PREFIX}{rank:08}/{ns}")
}

/// Creates a view doc and enrols it in this node's sync set.
///
/// Creating a doc opens it but does not make it syncable: a namespace absent
/// from the sync set rejects every incoming request with `NotFound`, so a
/// published view would be unreadable by the clients it exists for. An empty
/// peer list enrols without dialing anyone, which is what a publisher wants —
/// readers come to it.
async fn create_view(docs: &Docs) -> anyhow::Result<NamespaceId> {
    let doc = docs.api().create().await?;
    doc.start_sync(Vec::new()).await?;
    Ok(doc.id())
}

impl Views {
    pub async fn create(docs: &Docs) -> anyhow::Result<Self> {
        let recent = create_view(docs).await?;
        let featured = create_view(docs).await?;
        let categories = create_view(docs).await?;
        let active = create_view(docs).await?;

        Ok(Self {
            ids: ViewIds {
                recent,
                featured,
                categories,
                active,
            },
        })
    }

    /// Publishes which spaces have recent activity, most recent first.
    ///
    /// Individual heartbeats stay in memory — one entry per peer per space,
    /// rewritten every couple of minutes, is exactly what should not be in a
    /// synced doc. What clients need to *discover* a space is one entry per
    /// space, bounded by how many are active and ordered so a reader can take
    /// the first N.
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

    /// Recomputes every view from the catalog.
    ///
    /// Views are rewritten wholesale rather than patched: they are small and
    /// bounded by construction, and a full rebuild cannot drift from the
    /// catalog the way incremental edits can.
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
