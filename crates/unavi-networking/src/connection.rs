pub trait Connection {
    async fn new(addr: &str) -> impl Connection {
        #[cfg(not(target_family = "wasm"))]
        let conn = crate::native::connect(addr);

        #[cfg(target_family = "wasm")]
        let conn = crate::web::new_connection(addr);

        conn
    }
}
