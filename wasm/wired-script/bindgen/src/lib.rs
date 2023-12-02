wasmtime::component::bindgen!({
    world: "host",
    path: "../wit"
});

wasmtime::component::bindgen!({
    world: "script",
    path: "../wit"
});
