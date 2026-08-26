use anyhow::{
    Context,
    bail,
};
use iroh::{
    EndpointId,
    endpoint::{
        RecvStream,
        SendStream,
    },
};
use rand::RngCore;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};
use xdid::core::did::Did;

use crate::{
    identity::Identity,
    resolve::Resolver,
    signed_bytes::{
        Signable,
        SignedBytes,
    },
};

const MAX_PROOF_LEN: usize = 8 * 1024;
const ACCEPTED: u8 = 1;

pub type Nonce = [u8; 32];

/// A peer's claim to a DID, bound to the one connection it was made on.
///
/// The signature covers both endpoints, blocking proof theft (`prover`) and
/// relaying (`verifier`); the nonce is answered once, so nothing is
/// replayable.
#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityProof {
    pub did:      Did,
    pub prover:   EndpointId,
    pub verifier: EndpointId,
    pub nonce:    Nonce,
}

impl Signable for IdentityProof {
    const SIGNING_CONTEXT: &'static str = "wired/auth/identity";
}

pub struct Handshake<'a> {
    pub identity: &'a Identity,
    pub resolver: &'a Resolver,
    pub local:    EndpointId,
    pub remote:   EndpointId,
}

impl Handshake<'_> {
    /// Challenges the remote, then answers its counter-challenge, returning the
    /// DID the remote proved.
    pub async fn dial(&self, tx: &mut SendStream, rx: &mut RecvStream) -> anyhow::Result<Did> {
        let nonce = fresh_nonce();
        tx.write_all(&nonce).await.context("write nonce")?;

        let signed = read_proof(rx).await?;
        let remote_did = self.verified_did(&signed, nonce).await?;

        let mut counter = Nonce::default();
        rx.read_exact(&mut counter).await.context("read counter")?;
        self.write_proof(tx, counter).await?;
        tx.finish().context("finish proof stream")?;

        if rx.read_u8().await.context("read verdict")? != ACCEPTED {
            bail!("the remote refused our identity proof")
        }

        Ok(remote_did)
    }

    /// Answers the remote's challenge, then challenges it back, returning the
    /// DID it proved.
    pub async fn accept(&self, tx: &mut SendStream, rx: &mut RecvStream) -> anyhow::Result<Did> {
        let mut nonce = Nonce::default();
        rx.read_exact(&mut nonce).await.context("read nonce")?;
        self.write_proof(tx, nonce).await?;

        let counter = fresh_nonce();
        tx.write_all(&counter).await.context("write counter")?;

        let signed = read_proof(rx).await?;
        let remote_did = self.verified_did(&signed, counter).await?;

        tx.write_u8(ACCEPTED).await.context("write verdict")?;
        // A reset stream here means the remote never learned it was accepted,
        // so it will not bind us either.
        tx.finish().context("finish verdict stream")?;

        Ok(remote_did)
    }

    async fn write_proof(&self, tx: &mut SendStream, nonce: Nonce) -> anyhow::Result<()> {
        let signed = IdentityProof {
            did: self.identity.did().clone(),
            prover: self.local,
            verifier: self.remote,
            nonce,
        }
        .sign(self.identity.signing_key())
        .context("sign proof")?;

        let buf = postcard::to_allocvec(&signed)?;
        tx.write_u32(u32::try_from(buf.len())?).await?;
        tx.write_all(&buf).await.context("write proof")?;
        Ok(())
    }

    async fn verified_did(
        &self,
        signed: &SignedBytes<IdentityProof>,
        nonce: Nonce,
    ) -> anyhow::Result<Did> {
        let proof = signed.payload().context("parse proof")?;

        if proof.prover != self.remote {
            bail!("proof was made by {}, not {}", proof.prover, self.remote)
        }
        if proof.verifier != self.local {
            bail!(
                "proof was addressed to {}, not {}",
                proof.verifier,
                self.local
            )
        }
        if proof.nonce != nonce {
            bail!("proof answers a different challenge")
        }
        signed.verify(&proof.did, self.resolver).await?;

        Ok(proof.did)
    }
}

fn fresh_nonce() -> Nonce {
    let mut nonce = Nonce::default();
    rand::rng().fill_bytes(&mut nonce);
    nonce
}

async fn read_proof(rx: &mut RecvStream) -> anyhow::Result<SignedBytes<IdentityProof>> {
    let len = rx.read_u32().await.context("read proof len")? as usize;
    if len > MAX_PROOF_LEN {
        bail!("proof too large")
    }
    let mut buf = vec![0; len];
    rx.read_exact(&mut buf).await.context("read proof")?;
    postcard::from_bytes(&buf).context("parse proof")
}

#[cfg(test)]
mod tests {
    use iroh::SecretKey;
    use xdid::methods::key::keys::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    };

    use super::*;

    fn endpoint() -> EndpointId {
        SecretKey::generate().public()
    }

    fn signed(
        key: &P256KeyPair,
        did: Did,
        prover: EndpointId,
        verifier: EndpointId,
        nonce: Nonce,
    ) -> SignedBytes<IdentityProof> {
        IdentityProof {
            did,
            prover,
            verifier,
            nonce,
        }
        .sign(key)
        .expect("sign proof")
    }

    /// The DID is `did:key`, so verification resolves without any network.
    fn handshake<'a>(
        identity: &'a Identity,
        resolver: &'a Resolver,
        prover: EndpointId,
        verifier: EndpointId,
    ) -> Handshake<'a> {
        Handshake {
            identity,
            resolver,
            local: verifier,
            remote: prover,
        }
    }

    fn identity() -> Identity {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();
        Identity::new(did, key)
    }

    #[tokio::test]
    async fn a_proof_for_this_connection_verifies() {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();
        let (prover, verifier, nonce) = (endpoint(), endpoint(), [7u8; 32]);
        let (local, resolver) = (identity(), Resolver::new().expect("resolver"));

        let proof = signed(&key, did.clone(), prover, verifier, nonce);

        assert_eq!(
            handshake(&local, &resolver, prover, verifier)
                .verified_did(&proof, nonce)
                .await
                .expect("a well-formed proof must verify"),
            did
        );
    }

    #[tokio::test]
    async fn a_proof_cannot_be_relayed_to_another_verifier() {
        let key = P256KeyPair::generate();
        let (prover, verifier, nonce) = (endpoint(), endpoint(), [7u8; 32]);
        let (local, resolver) = (identity(), Resolver::new().expect("resolver"));

        let proof = signed(&key, key.public().to_did(), prover, verifier, nonce);

        assert!(
            handshake(&local, &resolver, prover, endpoint())
                .verified_did(&proof, nonce)
                .await
                .is_err(),
            "a proof addressed to one endpoint must not verify at another"
        );
    }

    #[tokio::test]
    async fn a_proof_cannot_be_presented_for_another_prover() {
        let key = P256KeyPair::generate();
        let (prover, verifier, nonce) = (endpoint(), endpoint(), [7u8; 32]);
        let (local, resolver) = (identity(), Resolver::new().expect("resolver"));

        let proof = signed(&key, key.public().to_did(), prover, verifier, nonce);

        assert!(
            handshake(&local, &resolver, endpoint(), verifier)
                .verified_did(&proof, nonce)
                .await
                .is_err(),
            "a proof made by one endpoint must not identify another"
        );
    }

    #[tokio::test]
    async fn a_proof_answering_a_different_nonce_is_refused() {
        let key = P256KeyPair::generate();
        let (prover, verifier) = (endpoint(), endpoint());
        let (local, resolver) = (identity(), Resolver::new().expect("resolver"));

        let proof = signed(&key, key.public().to_did(), prover, verifier, [7u8; 32]);

        assert!(
            handshake(&local, &resolver, prover, verifier)
                .verified_did(&proof, [8u8; 32])
                .await
                .is_err(),
            "a replayed proof must not answer a fresh challenge"
        );
    }

    #[tokio::test]
    async fn a_did_the_signer_does_not_control_is_refused() {
        let (key, other) = (P256KeyPair::generate(), P256KeyPair::generate());
        let (prover, verifier, nonce) = (endpoint(), endpoint(), [7u8; 32]);
        let (local, resolver) = (identity(), Resolver::new().expect("resolver"));

        let proof = signed(&key, other.public().to_did(), prover, verifier, nonce);

        assert!(
            handshake(&local, &resolver, prover, verifier)
                .verified_did(&proof, nonce)
                .await
                .is_err(),
            "claiming a DID must require holding its key"
        );
    }
}
