use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "protos/webrtc/common.proto",
            "protos/webrtc/request.proto",
            "protos/webrtc/response.proto",
            "protos/websocket/request.proto",
            "protos/websocket/response.proto",
            "protos/websocket/signaling/common.proto",
            "protos/websocket/signaling/request.proto",
            "protos/websocket/signaling/response.proto",
        ],
        &["protos/", "src/"],
    )?;
    Ok(())
}
