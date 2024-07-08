mod player;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-player",
    world: "host",
    with: {
        "wired:player/api/player": player::Player,
        "wired:scene/node/node": crate::api::wired_scene::gltf::node::Node,
    }
});
