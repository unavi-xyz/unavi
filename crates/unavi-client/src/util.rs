pub fn new_tokio_runtime(thread_name: &str) -> tokio::runtime::Runtime {
    #[cfg(not(target_family = "wasm"))]
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(thread_name)
        .build()
        .expect("build tokio runtime");

    #[cfg(target_family = "wasm")]
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name(thread_name)
        .build()
        .expect("build tokio runtime");

    rt
}
