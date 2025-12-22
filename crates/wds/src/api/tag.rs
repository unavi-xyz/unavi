use std::{fmt::Display, str::FromStr};

use blake3::Hash;
use xdid::core::did::Did;

pub struct BlobTag {
    owner: Did,
    hash: Hash,
}

const DELIM: char = '_';

impl Display for BlobTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}{DELIM}{}", self.owner, self.hash))
    }
}

impl FromStr for BlobTag {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, DELIM);
        let owner_str = parts.next().ok_or_else(|| anyhow::anyhow!("no owner"))?;
        let hash_str = parts.next().ok_or_else(|| anyhow::anyhow!("no hash"))?;

        let owner = Did::from_str(owner_str)?;
        let hash = Hash::from_str(hash_str)?;

        Ok(Self { owner, hash })
    }
}

impl BlobTag {
    pub const fn new(owner: Did, hash: Hash) -> Self {
        Self { owner, hash }
    }
}
