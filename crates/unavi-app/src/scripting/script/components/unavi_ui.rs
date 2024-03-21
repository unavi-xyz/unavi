wasm_bridge::component::bindgen!({
    async: true,
    path: "../../components/unavi-ui/wit/world.wit",
    world: "imports",
});

struct MyStruct;

impl self::unavi::ui::api::Host for MyStruct {
    fn new_bubble<'life0, 'async_trait>(
        &'life0 mut self,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = wasm_bridge::Result<String>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { Ok("".to_string()) })
    }
}
