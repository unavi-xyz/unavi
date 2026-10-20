//! `wired/profile`: a peer answers with the proof of its user's profile.

use std::{
    sync::Arc,
    time::Duration,
};

use iroh::{
    Endpoint,
    EndpointId,
    endpoint::{
        ClosedStream,
        ConnectError,
        Connection,
        ConnectionError,
        ReadToEndError,
        VarInt,
        WriteError,
    },
    protocol::{
        AcceptError,
        ProtocolHandler,
    },
};
use iroh_docs::NamespaceId;
use tracing::debug;
use xdid::core::did::Did;

use crate::{
    authorship::{
        Authorship,
        AuthorshipError,
        MAX_AUTHORSHIP_BYTES,
    },
    resolver::Resolver,
};

pub const ALPN: &[u8] = b"wired/profile";

/// Bounds an ask, and how long an answer holds its connection open for the
/// asker to read it.
const DEADLINE: Duration = Duration::from_secs(10);
const CLOSE_DONE: u32 = 0;

#[derive(Debug, thiserror::Error)]
pub enum AskError {
    #[error(transparent)]
    Connect(#[from] ConnectError),
    #[error(transparent)]
    Connection(#[from] ConnectionError),
    #[error(transparent)]
    Read(#[from] ReadToEndError),
    #[error("the peer did not answer in time")]
    TimedOut,
    /// The peer answered with a proof that does not hold for the DID asked
    /// about.
    #[error(transparent)]
    Proof(#[from] AuthorshipError),
}

/// Serves the local user's profile proof. Registered on the router under
/// [`ALPN`].
#[derive(Clone, Debug)]
pub struct ProfileProtocol {
    proof: Arc<[u8]>,
}

impl ProfileProtocol {
    pub fn new(proof: &Authorship) -> postcard::Result<Self> {
        Ok(Self {
            proof: postcard::to_stdvec(proof)?.into(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
enum AnswerError {
    #[error(transparent)]
    Connection(#[from] ConnectionError),
    #[error(transparent)]
    Write(#[from] WriteError),
    #[error(transparent)]
    Closed(#[from] ClosedStream),
}

impl ProtocolHandler for ProfileProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let answer = async {
            let mut tx = connection.open_uni().await?;
            tx.write_all(&self.proof).await?;
            tx.finish()?;
            Ok::<_, AnswerError>(())
        };
        match n0_future::time::timeout(DEADLINE, answer).await {
            Ok(Ok(())) => {
                if n0_future::time::timeout(DEADLINE, connection.closed())
                    .await
                    .is_err()
                {
                    connection.close(VarInt::from_u32(CLOSE_DONE), b"done");
                }
            }
            Ok(Err(err)) => debug!(?err, "profile answer failed"),
            Err(_) => {
                debug!(remote = %connection.remote_id(), "profile answer timed out");
                connection.close(VarInt::from_u32(CLOSE_DONE), b"timed out");
            }
        }
        Ok(())
    }
}

/// The profile namespace of `did`, asked of `peer`, an endpoint that speaks
/// for it.
pub async fn ask(
    endpoint: &Endpoint,
    peer: EndpointId,
    did: &Did,
    resolver: &Resolver,
) -> Result<NamespaceId, AskError> {
    let read = async {
        let connection = endpoint.connect(peer, ALPN).await?;
        let mut rx = connection.accept_uni().await?;
        let bytes = rx.read_to_end(MAX_AUTHORSHIP_BYTES).await?;
        connection.close(VarInt::from_u32(CLOSE_DONE), b"done");
        Ok::<_, AskError>(bytes)
    };
    let bytes = n0_future::time::timeout(DEADLINE, read)
        .await
        .map_err(|_| AskError::TimedOut)??;
    let proof: Authorship = postcard::from_bytes(&bytes).map_err(|_| AuthorshipError::Malformed)?;
    Ok(proof.verify_for(did, resolver).await?)
}
