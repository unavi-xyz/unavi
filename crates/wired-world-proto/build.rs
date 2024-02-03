use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &[
            "../../wired-protocol/world/protos/webrtc/common.proto",
            "../../wired-protocol/world/protos/webrtc/request.proto",
            "../../wired-protocol/world/protos/webrtc/response.proto",
            "../../wired-protocol/world/protos/websocket/request.proto",
            "../../wired-protocol/world/protos/websocket/response.proto",
            "../../wired-protocol/world/protos/websocket/signaling/common.proto",
            "../../wired-protocol/world/protos/websocket/signaling/request.proto",
            "../../wired-protocol/world/protos/websocket/signaling/response.proto",
        ],
        &["../../wired-protocol/world/protos/", "src/"],
    )?;
    Ok(())
}
