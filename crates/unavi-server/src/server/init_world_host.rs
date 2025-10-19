use dwn::Actor;
use unavi_server_service::Version;

const WP_PREFIX: &str = "https://wired-protocol.org/v0/";
const WP_VERSION: Version = Version::new(0, 1, 0);

const HOST_DEFINITION: &[u8] = include_bytes!("../../../../protocol/dwn/protocols/world-host.json");

pub async fn init_world_host(actor: &Actor) -> anyhow::Result<()> {
    let host_def = serde_json::from_slice(HOST_DEFINITION)?;

    actor
        .configure_protocol(WP_VERSION, host_def)
        .process()
        .await?;

    Ok(())
}
