use wired_world::world_server_capnp::world_server::Client;

pub async fn tickrate(rpc: &Client) -> Result<f32, capnp::Error> {
    let request = rpc.tickrate_request();

    let reply = request.send().promise.await?;
    let tickrate = reply.get()?.get_tickrate();

    Ok(tickrate)
}
