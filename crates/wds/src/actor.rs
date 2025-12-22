use xdid::{core::did::Did, methods::key::p256::P256KeyPair};

pub struct Actor {
    did: Did,
    signing_key: P256KeyPair,
}

impl Actor {
    #[must_use]
    pub const fn did(&self) -> &Did {
        &self.did
    }
}
