use super::ServerOptions;

use unavi_server_world::WorldOptions;

pub async fn start(opts: &ServerOptions) -> Result<(), Box<dyn std::error::Error>> {
    let identity = unavi_server_world::cert::generate_tls_identity();

    unavi_server_world::start_server(WorldOptions {
        address: opts.world_addr,
        identity: &identity,
    })
    .await?;

    Ok(())
}
