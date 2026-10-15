//! Writing the views clients sync.
//!
//! Each write diffs against what the view holds, so a syncing client never
//! sees a view emptied and refilled, and an unchanged entry is not rewritten.

use std::collections::BTreeMap;

use iroh_docs::NamespaceId;
use unavi_store::{
    Document,
    Store,
};

use crate::{
    claim::Submission,
    server::{
        config::Config,
        presence::ActiveSpace,
    },
    views::{
        ACTIVE_PREFIX,
        ViewIds,
    },
};

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

fn active_value(space: &ActiveSpace) -> anyhow::Result<Vec<u8>> {
    let occupants = u32::try_from(space.occupants).unwrap_or(u32::MAX);
    let idle_secs = space.idle_secs / 60 * 60;
    Ok(postcard::to_stdvec(&(
        occupants,
        idle_secs,
        space.title.as_str(),
    ))?)
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

    #[must_use]
    pub fn ids(&self) -> ViewIds {
        ViewIds {
            recent:     self.recent.id(),
            featured:   self.featured.id(),
            categories: self.categories.id(),
            active:     self.active.id(),
        }
    }

    /// Idle time is published in whole minutes, so an unchanged space is
    /// rewritten at most once a minute.
    pub async fn write_active(
        &self,
        active: &[ActiveSpace],
        capacity: usize,
    ) -> anyhow::Result<()> {
        let mut entries = BTreeMap::new();
        for (rank, space) in active.iter().take(capacity).enumerate() {
            entries.insert(active_key(rank, space.ns), active_value(space)?);
        }
        sync(&self.active, ACTIVE_PREFIX, entries).await
    }

    /// Rewrites the listing views from `live`.
    pub async fn rebuild(&self, mut live: Vec<Submission>, config: &Config) -> anyhow::Result<()> {
        live.sort_by_key(|s| std::cmp::Reverse(s.expires));

        let recent = live.iter().take(config.view_capacity);
        sync(&self.recent, "", ranked(recent, ranked_key)?).await?;

        let featured = live
            .iter()
            .filter(|s| config.featured.contains(&s.ns))
            .take(config.view_capacity);
        sync(&self.featured, "", ranked(featured, ranked_key)?).await?;

        let mut categories = BTreeMap::new();
        for category in &config.categories {
            let matching = live
                .iter()
                .filter(|s| s.tags.iter().any(|t| t == category))
                .take(config.view_capacity);
            categories.extend(ranked(matching, |rank, ns| {
                category_key(category, rank, ns)
            })?);
        }
        sync(&self.categories, "", categories).await
    }
}

fn ranked<'a>(
    submissions: impl Iterator<Item = &'a Submission>,
    key: impl Fn(usize, NamespaceId) -> String,
) -> anyhow::Result<BTreeMap<String, Vec<u8>>> {
    submissions
        .enumerate()
        .map(|(rank, submission)| Ok((key(rank, submission.ns), postcard::to_stdvec(submission)?)))
        .collect()
}

/// Makes `view`'s keys under `prefix` exactly `entries`: changed keys are set,
/// vanished ones removed, and the rest left alone.
async fn sync(
    view: &Document,
    prefix: &str,
    mut entries: BTreeMap<String, Vec<u8>>,
) -> anyhow::Result<()> {
    for entry in view.list(prefix).await? {
        let key = String::from_utf8_lossy(entry.key()).into_owned();
        match entries.get(&key) {
            Some(value) if view.value(&entry).await?.as_deref() == Some(value.as_slice()) => {
                entries.remove(&key);
            }
            Some(_) => {}
            None => view.remove_key(key).await?,
        }
    }

    for (key, value) in entries {
        view.set(key, value).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_active_value_carries_occupancy_whole_minutes_and_the_title() {
        let space = ActiveSpace {
            ns:        NamespaceId::from(&[1; 32]),
            occupants: 3,
            idle_secs: 125,
            title:     "Plaza".into(),
        };

        let value = active_value(&space).expect("encode");
        let decoded: (u32, u64, String) = postcard::from_bytes(&value).expect("decode");

        assert_eq!(decoded, (3, 120, "Plaza".to_owned()));
    }
}
