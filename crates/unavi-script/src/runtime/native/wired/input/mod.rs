pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-input",
        with: {
            "wired:scene/types.node": HostNode,
            "wired:input/types.input-listener": HostInputListener,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}
