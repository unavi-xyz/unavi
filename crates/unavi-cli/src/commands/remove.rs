use anyhow::Result;
use dwn::Actor;

pub async fn remove_space(_actor: &Actor, _id: &str) -> Result<()> {
    println!("Removing space... (not implemented yet)");
    Ok(())
}
