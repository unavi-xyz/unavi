use anyhow::Result;
use dwn::Actor;

pub async fn list_spaces(_actor: &Actor) -> Result<()> {
    println!("Listing spaces... (not implemented yet)");
    Ok(())
}
