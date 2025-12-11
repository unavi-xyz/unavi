use std::str::FromStr;

use anyhow::Result;
use dwn::Actor;
use unavi_constants::SPACE_HOST_DID;
use xdid::core::did::Did;

pub async fn remove_space(actor: &Actor, id: String) -> Result<()> {
    let space_host = Did::from_str(SPACE_HOST_DID)?;

    actor.delete(id).target(&space_host).send_remote().await?;

    Ok(())
}
