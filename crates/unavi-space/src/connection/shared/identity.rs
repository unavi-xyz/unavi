use std::time::Duration;

use anyhow::{
    Context,
    bail,
};
use iroh::{
    EndpointId,
    endpoint::{
        Connection,
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
use tracing::{
    debug,
    info,
    warn,
};
use unavi_policy::identity;
use wds::signed_bytes::{
    Signable,
    SignedBytes,
    verify_did_signature,
};
use xdid::core::did::Did;

use crate::{
    connection::shared::StreamIdent,
    peer::{
        self_endpoint_id,
        self_identity,
    },
};

const MAX_PROOF_LEN: usize = 8 * 1024;
const PROOF_ATTEMPTS: u32 = 5;
const PROOF_RETRY_DELAY: Duration = Duration::from_secs(1);
/// Generous, since answering means resolving a DID that may be `did:web`.
const PROOF_TIMEOUT: Duration = Duration::from_secs(15);

type Nonce = [u8; 32];

/// A peer's claim to a DID, bound to the one connection it was made on. The
/// signature covers both endpoints, blocking proof theft (`prover`) and
/// relaying (`verifier`); the nonce is answered once, so nothing is
/// replayable.
#[derive(Debug, Serialize, Deserialize)]
struct PeerBinding {
    did:      Did,
    prover:   EndpointId,
    verifier: EndpointId,
    nonce:    Nonce,
}

impl Signable for PeerBinding {
    const SIGNING_CONTEXT: &'static str = "unavi/space/identity";
}

/// Challenges the peer to prove which DID it speaks for, binding the result to
/// this connection for as long as it lasts. Retried because a peer whose own
/// identity is still loading cannot answer yet.
pub async fn verify_peer_identity(connection: &Connection) {
    let peer = connection.remote_id();

    for attempt in 1..=PROOF_ATTEMPTS {
        // A peer that accepts the stream and never answers would otherwise
        // hold the attempt open for as long as the connection lives.
        let proof = n0_future::time::timeout(PROOF_TIMEOUT, request_proof(connection))
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("identity proof timed out")));

        match proof {
            Ok(did) => {
                info!("Peer identified as {did}");
                // Bound before the block is judged, so a refusal has a DID to
                // key its teardown to.
                identity::bind(*peer.as_bytes(), did.clone());
                if crate::trust::enforce_block(peer, &did) {
                    identity::unbind(*peer.as_bytes());
                }
                return;
            }
            Err(err) if attempt == PROOF_ATTEMPTS => warn!(?err, "Peer proved no identity"),
            Err(err) => {
                debug!(?err, "Identity proof failed, retrying");
                n0_future::time::sleep(PROOF_RETRY_DELAY).await;
            }
        }
    }
}

async fn request_proof(connection: &Connection) -> anyhow::Result<Did> {
    let peer = connection.remote_id();
    let verifier = self_endpoint_id()?;

    let (mut tx, mut rx) = connection.open_bi().await.context("open_bi")?;
    StreamIdent::Identity.write(&mut tx).await?;

    let mut nonce = Nonce::default();
    rand::rng().fill_bytes(&mut nonce);
    tx.write_all(&nonce).await.context("write nonce")?;
    let _ = tx.finish();

    let len = rx.read_u32().await.context("read proof len")? as usize;
    if len > MAX_PROOF_LEN {
        bail!("proof too large")
    }
    let mut buf = vec![0; len];
    rx.read_exact(&mut buf).await.context("read proof")?;

    let signed = postcard::from_bytes::<SignedBytes<PeerBinding>>(&buf).context("parse proof")?;
    verified_did(&signed, peer, verifier, nonce).await
}

/// Answers the peer's challenge with a signed claim to the local DID.
pub async fn prove_self_identity(
    peer: EndpointId,
    mut tx: SendStream,
    mut rx: RecvStream,
) -> anyhow::Result<()> {
    let Some(identity) = self_identity() else {
        bail!("no local identity to prove")
    };

    let mut nonce = Nonce::default();
    rx.read_exact(&mut nonce).await.context("read nonce")?;

    let binding = PeerBinding {
        did: identity.did().clone(),
        prover: self_endpoint_id()?,
        verifier: peer,
        nonce,
    };

    let signed = binding
        .sign(identity.signing_key())
        .context("sign binding")?;
    let buf = postcard::to_allocvec(&signed)?;

    tx.write_u32(u32::try_from(buf.len())?).await?;
    tx.write_all(&buf).await.context("write proof")?;
    let _ = tx.finish();

    Ok(())
}

/// The DID a proof establishes, or an error naming why it establishes nothing.
async fn verified_did(
    signed: &SignedBytes<PeerBinding>,
    prover: EndpointId,
    verifier: EndpointId,
    nonce: Nonce,
) -> anyhow::Result<Did> {
    let binding = signed.payload().context("parse binding")?;

    if binding.prover != prover {
        bail!("proof was made by {}, not {prover}", binding.prover)
    }
    if binding.verifier != verifier {
        bail!(
            "proof was addressed to {}, not {verifier}",
            binding.verifier
        )
    }
    if binding.nonce != nonce {
        bail!("proof answers a different challenge")
    }
    if !verify_did_signature(signed, &binding.did).await {
        bail!("proof is not signed by {}", binding.did)
    }

    Ok(binding.did)
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
    ) -> SignedBytes<PeerBinding> {
        PeerBinding {
            did,
            prover,
            verifier,
            nonce,
        }
        .sign(key)
        .expect("sign binding")
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
