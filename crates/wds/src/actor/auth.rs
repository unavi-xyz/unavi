use crate::{
    SessionToken,
    actor::Actor,
    auth::client::authenticate,
};

impl Actor {
    /// The actor's session at its host, establishing one if needed.
    pub async fn session(&self) -> anyhow::Result<SessionToken> {
        self.authenticate().await
    }

    pub(crate) async fn authenticate(&self) -> anyhow::Result<SessionToken> {
        let session = self.session.lock().await;

        // Lock held through authentication so concurrent callers share one
        // handshake.
        if let Some(s) = session.get().copied() {
            return Ok(s);
        }

        let s = authenticate(
            self.identity().did().clone(),
            self.identity().signing_key(),
            self.host.id,
            &self.auth_client,
        )
        .await?;

        session.set(s)?;
        drop(session);

        Ok(s)
    }
}
