use anyhow::{Context, bail};
use iroh::EndpointId;
use irpc::Client;
use xdid::{core::did::Did, methods::key::Signer};

use crate::{
    SessionToken,
    auth::{AnswerChallenge, AuthService, Challenge, RequestChallenge},
    signed_bytes::Signable,
};

pub async fn authenticate<S>(
    did: Did,
    signer: &S,
    host: EndpointId,
    auth_client: &Client<AuthService>,
) -> anyhow::Result<SessionToken>
where
    S: Signer + Sync,
{
    let nonce = auth_client
        .rpc(RequestChallenge(did.clone()))
        .await
        .context("request challenge")?;

    let challenge = Challenge { did, host, nonce };

    let payload = challenge.sign(signer).context("sign challenge")?;

    let Some(s) = auth_client
        .rpc(AnswerChallenge(payload))
        .await
        .context("answer challenge rpc")?
    else {
        bail!("challenge answer failed")
    };

    Ok(s)
}
