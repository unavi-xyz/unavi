use super::ServerOptions;

use unavi_server_world::WorldOptions;

pub async fn start(opts: &ServerOptions) -> Result<(), Box<dyn std::error::Error>> {
    let certificate = unavi_server_world::cert::generate_certificate();

    unavi_server_world::start_server(WorldOptions {
        address: opts.world_addr,
        certificate,
    })
    .await?;

    Ok(())
}
