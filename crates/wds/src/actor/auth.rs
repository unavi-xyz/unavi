use anyhow::{Context, bail};
use tracing::debug;

use crate::{
    SessionToken,
    actor::Actor,
    auth::{AnswerChallenge, Challenge, RequestChallenge},
    signed_bytes::Signable,
};

impl Actor {
    pub(crate) async fn authenticate(&self) -> anyhow::Result<SessionToken> {
        let session = self.session.lock().await;

        // If not authed, hold the lock while we authenticate.
        if let Some(s) = session.get().copied() {
            return Ok(s);
        }

        debug!("authenticating");

        let nonce = self
            .auth_client
            .rpc(RequestChallenge(self.did.clone()))
            .await
            .context("request challenge")?;

        let challenge = Challenge {
            did: self.did.clone(),
            host: self.host,
            nonce,
        };

        let signed = challenge
            .sign(&self.signing_key)
            .context("sign challenge")?;

        let Some(s) = self
            .auth_client
            .rpc(AnswerChallenge(signed))
            .await
            .context("answer challenge rpc")?
        else {
            bail!("failed to authenticate")
        };

        session.set(s)?;
        drop(session);

        debug!("successfully authenticated");

        Ok(s)
    }
}
