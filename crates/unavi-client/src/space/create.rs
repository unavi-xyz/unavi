use wired_data_store::Actor;

pub async fn create_space(actor: &Actor) -> anyhow::Result<()> {
    let id = actor.create_record(None).await?;

    actor
        .update_record(&id, |doc| {
            let map = doc.get_map("space");
            map.insert("name", format!("{}'s Space", actor.did()))?;
            Ok(())
        })
        .await?;

    Ok(())
}
