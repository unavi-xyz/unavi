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
    identity,
    signed_bytes::{
        Signable,
        SignedBytes,
        verify_did_signature,
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

/// Runs the dialing half: challenge the remote, then answer its counter-
/// challenge, returning the DID the remote proved.
pub async fn dial(
    tx: &mut SendStream,
    rx: &mut RecvStream,
    local: EndpointId,
    remote: EndpointId,
) -> anyhow::Result<Did> {
    let nonce = fresh_nonce();
    tx.write_all(&nonce).await.context("write nonce")?;

    let signed = read_proof(rx).await?;
    let remote_did = verified_did(&signed, remote, local, nonce).await?;

    let mut counter = Nonce::default();
    rx.read_exact(&mut counter).await.context("read counter")?;
    write_proof(tx, local, remote, counter).await?;
    let _ = tx.finish();

    if rx.read_u8().await.context("read verdict")? != ACCEPTED {
        bail!("the remote refused our identity proof")
    }

    Ok(remote_did)
}

/// Runs the accepting half: answer the remote's challenge, then challenge it
/// back, returning the DID it proved.
pub async fn accept(
    tx: &mut SendStream,
    rx: &mut RecvStream,
    local: EndpointId,
    remote: EndpointId,
) -> anyhow::Result<Did> {
    let mut nonce = Nonce::default();
    rx.read_exact(&mut nonce).await.context("read nonce")?;
    write_proof(tx, local, remote, nonce).await?;

    let counter = fresh_nonce();
    tx.write_all(&counter).await.context("write counter")?;

    let signed = read_proof(rx).await?;
    let remote_did = verified_did(&signed, remote, local, counter).await?;

    tx.write_u8(ACCEPTED).await.context("write verdict")?;
    let _ = tx.finish();

    Ok(remote_did)
}

fn fresh_nonce() -> Nonce {
    let mut nonce = Nonce::default();
    rand::rng().fill_bytes(&mut nonce);
    nonce
}

async fn write_proof(
    tx: &mut SendStream,
    prover: EndpointId,
    verifier: EndpointId,
    nonce: Nonce,
) -> anyhow::Result<()> {
    let identity = identity::local().context("no local identity to prove")?;

    let signed = IdentityProof {
        did: identity.did().clone(),
        prover,
        verifier,
        nonce,
    }
    .sign(identity.signing_key())
    .context("sign proof")?;

    let buf = postcard::to_allocvec(&signed)?;
    tx.write_u32(u32::try_from(buf.len())?).await?;
    tx.write_all(&buf).await.context("write proof")?;
    Ok(())
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

/// The DID a proof establishes, or an error naming why it establishes nothing.
async fn verified_did(
    signed: &SignedBytes<IdentityProof>,
    prover: EndpointId,
    verifier: EndpointId,
    nonce: Nonce,
) -> anyhow::Result<Did> {
    let proof = signed.payload().context("parse proof")?;

    if proof.prover != prover {
        bail!("proof was made by {}, not {prover}", proof.prover)
    }
    if proof.verifier != verifier {
        bail!("proof was addressed to {}, not {verifier}", proof.verifier)
    }
    if proof.nonce != nonce {
        bail!("proof answers a different challenge")
    }
    if !verify_did_signature(signed, &proof.did).await {
        bail!("proof is not signed by {}", proof.did)
    }

    Ok(proof.did)
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

    #[tokio::test]
    async fn a_proof_for_this_connection_verifies() {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();
        let (prover, verifier, nonce) = (endpoint(), endpoint(), [7u8; 32]);

        let proof = signed(&key, did.clone(), prover, verifier, nonce);

        assert_eq!(
            verified_did(&proof, prover, verifier, nonce)
                .await
                .expect("a well-formed proof must verify"),
            did
        );
    }

    #[tokio::test]
    async fn a_proof_cannot_be_relayed_to_another_verifier() {
        let key = P256KeyPair::generate();
        let (prover, verifier, nonce) = (endpoint(), endpoint(), [7u8; 32]);

        let proof = signed(&key, key.public().to_did(), prover, verifier, nonce);

        assert!(
            verified_did(&proof, prover, endpoint(), nonce)
                .await
                .is_err(),
            "a proof addressed to one endpoint must not verify at another"
        );
    }

    #[tokio::test]
    async fn a_proof_cannot_be_presented_for_another_prover() {
        let key = P256KeyPair::generate();
        let (prover, verifier, nonce) = (endpoint(), endpoint(), [7u8; 32]);

        let proof = signed(&key, key.public().to_did(), prover, verifier, nonce);

        assert!(
            verified_did(&proof, endpoint(), verifier, nonce)
                .await
                .is_err(),
            "a proof made by one endpoint must not identify another"
        );
    }

    #[tokio::test]
    async fn a_proof_answering_a_different_nonce_is_refused() {
        let key = P256KeyPair::generate();
        let (prover, verifier) = (endpoint(), endpoint());

        let proof = signed(&key, key.public().to_did(), prover, verifier, [7u8; 32]);

        assert!(
            verified_did(&proof, prover, verifier, [8u8; 32])
                .await
                .is_err(),
            "a replayed proof must not answer a fresh challenge"
        );
    }

    #[tokio::test]
    async fn a_did_the_signer_does_not_control_is_refused() {
        let (key, other) = (P256KeyPair::generate(), P256KeyPair::generate());
        let (prover, verifier, nonce) = (endpoint(), endpoint(), [7u8; 32]);

        let proof = signed(&key, other.public().to_did(), prover, verifier, nonce);

        assert!(
            verified_did(&proof, prover, verifier, nonce).await.is_err(),
            "claiming a DID must require holding its key"
        );
    }
}
