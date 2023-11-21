use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "protos/world/webrtc/common.proto",
            "protos/world/webrtc/request.proto",
            "protos/world/webrtc/response.proto",
            "protos/world/websocket/request.proto",
            "protos/world/websocket/response.proto",
            "protos/world/websocket/signaling/common.proto",
            "protos/world/websocket/signaling/request.proto",
            "protos/world/websocket/signaling/response.proto",
        ],
        &["protos/", "src/"],
    )?;
    Ok(())
}
