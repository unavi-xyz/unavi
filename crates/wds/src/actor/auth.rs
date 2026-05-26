use crate::{SessionToken, actor::Actor};

impl Actor {
    pub(crate) async fn authenticate(&self) -> anyhow::Result<SessionToken> {
        let session = self.session.lock().await;

        // If not authed, hold the lock while we authenticate.
        if let Some(s) = session.get().copied() {
            return Ok(s);
        }

        let s = crate::auth::client::authenticate(
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
