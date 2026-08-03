use iroh_blobs::Hash;
use iroh_docs::{
    AuthorId,
    NamespaceId,
};
use xdid::core::did::Did;

pub const PROFILE: &str = "profile";
pub const HOME: &str = "home";

pub const KEYS_PREFIX: &str = "keys/";
pub const AVATARS_PREFIX: &str = "avatars/";
pub const SPACES_PREFIX: &str = "spaces/";

pub const SNAPSHOT: &str = "snapshot";
pub const LOG_PREFIX: &str = "log/";
pub const DEPS_PREFIX: &str = "deps/";

pub const BEACONS_PREFIX: &str = "beacons/";

#[must_use]
pub fn author_binding(author: AuthorId) -> String {
    format!("{KEYS_PREFIX}{author}")
}

#[must_use]
pub fn avatar(id: &str) -> String {
    format!("{AVATARS_PREFIX}{id}")
}

#[must_use]
pub fn space(id: &str) -> String {
    format!("{SPACES_PREFIX}{id}")
}

#[must_use]
pub fn dep(hash: Hash) -> String {
    format!("{DEPS_PREFIX}{}", hash.to_hex())
}

#[must_use]
pub fn beacon(space: NamespaceId, did: &Did) -> String {
    format!("{BEACONS_PREFIX}{space}/{did}")
}

#[must_use]
pub fn beacons_for_space(space: NamespaceId) -> String {
    format!("{BEACONS_PREFIX}{space}/")
}
