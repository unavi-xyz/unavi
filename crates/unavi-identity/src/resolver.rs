//! DID resolution, cached so a verification never costs more than one fetch per
//! DID per [`TTL`].

use std::{
    collections::HashMap,
    time::Duration,
};

use n0_future::time::Instant;
use parking_lot::Mutex;
use xdid::{
    core::{
        Method,
        ResolutionError,
        did::Did,
        document::Document,
    },
    method::{
        key::MethodDidKey,
        web::{
            ClientError,
            Config,
            MethodDidWeb,
            target::TargetPolicy,
        },
    },
    resolver::DidResolver,
};

/// Debug builds resolve a `did:web` naming a loopback or private host over
/// plaintext HTTP, so a development server is reachable. Release builds refuse
/// them, where resolving a peer-supplied DID would otherwise probe this host's
/// own network.
const TARGET: TargetPolicy = if cfg!(debug_assertions) {
    TargetPolicy::AllowLocal
} else {
    TargetPolicy::PublicOnly
};

/// How long a resolved document is reused. A key rotation takes this long to
/// reach a verifier.
pub const TTL: Duration = Duration::from_mins(5);

/// How long a failed resolution is remembered, so a DID whose host is down or
/// hostile is not fetched again on every request naming it.
const FAILURE_TTL: Duration = Duration::from_secs(30);

/// Most DIDs held at once. Past it the oldest entry is evicted.
const CAPACITY: usize = 4096;

/// Resolves `did:key` and `did:web`, caching each `did:web` answer.
///
/// `did:key` resolves locally and is never cached, so minting keys cannot
/// flush the cache.
#[derive(Debug)]
pub struct Resolver {
    inner: DidResolver,
    cache: Mutex<HashMap<Did, Cached>>,
}

#[derive(Debug)]
struct Cached {
    at:     Instant,
    answer: Result<Document, ResolutionError>,
}

impl Cached {
    fn fresh(&self, now: Instant) -> bool {
        let ttl = if self.answer.is_ok() {
            TTL
        } else {
            FAILURE_TTL
        };
        now.saturating_duration_since(self.at) < ttl
    }
}

impl Resolver {
    pub fn new() -> Result<Self, ClientError> {
        let web = MethodDidWeb::with_config(Config {
            target: TARGET,
            ..Config::default()
        })?;

        Ok(Self {
            inner: DidResolver::with_methods([
                Box::new(MethodDidKey) as Box<dyn Method>,
                Box::new(web),
            ]),
            cache: Mutex::default(),
        })
    }

    /// `did`'s document, from the cache while it is fresh.
    pub async fn resolve(&self, did: &Did) -> Result<Document, ResolutionError> {
        if did.method_name.as_str() == "key" {
            return self.inner.resolve(did).await;
        }

        let now = Instant::now();
        if let Some(cached) = self.cache.lock().get(did)
            && cached.fresh(now)
        {
            return cached.answer.as_ref().cloned().map_err(copy_error);
        }

        let answer = self.inner.resolve(did).await;
        let returned = answer.as_ref().cloned().map_err(copy_error);

        let mut cache = self.cache.lock();
        if cache.len() >= CAPACITY && !cache.contains_key(did) {
            evict(&mut cache, now);
        }
        cache.insert(did.clone(), Cached { at: now, answer });

        returned
    }
}

/// Drops every stale entry, or the oldest one if none is stale.
fn evict(cache: &mut HashMap<Did, Cached>, now: Instant) {
    cache.retain(|_, cached| cached.fresh(now));
    if cache.len() < CAPACITY {
        return;
    }
    if let Some(oldest) = cache
        .iter()
        .min_by_key(|(_, cached)| cached.at)
        .map(|(did, _)| did.clone())
    {
        cache.remove(&oldest);
    }
}

/// [`ResolutionError`] is not `Clone`; a cached failure is answered with an
/// equivalent one.
fn copy_error(err: &ResolutionError) -> ResolutionError {
    match err {
        ResolutionError::InvalidDid => ResolutionError::InvalidDid,
        ResolutionError::DocumentMismatch => ResolutionError::DocumentMismatch,
        ResolutionError::TargetNotAllowed => ResolutionError::TargetNotAllowed,
        ResolutionError::DocumentTooLarge => ResolutionError::DocumentTooLarge,
        ResolutionError::NotFound => ResolutionError::NotFound,
        ResolutionError::Malformed(detail) => ResolutionError::Malformed(detail.clone()),
        ResolutionError::UnsupportedMethod => ResolutionError::UnsupportedMethod,
        ResolutionError::Transport(detail) => ResolutionError::Transport(detail.clone()),
        other => ResolutionError::Transport(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn eviction_keeps_the_cache_bounded() {
        let now = Instant::now();
        let mut cache = HashMap::new();
        for i in 0..CAPACITY {
            let did = Did::from_str(&format!("did:web:{i}.example")).expect("did");
            cache.insert(
                did,
                Cached {
                    at:     now,
                    answer: Err(ResolutionError::NotFound),
                },
            );
        }

        evict(&mut cache, now);

        assert!(cache.len() < CAPACITY);
    }
}
