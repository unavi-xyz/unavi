use std::cell::Cell;

use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefResource},
    state::StoreState,
};

use super::wired::input::{handler::HostInputHandler, types::InputEvent};

pub struct InputHandler {
    ref_count: Cell<usize>,
}

impl RefCount for InputHandler {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for InputHandler {}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            ref_count: Cell::new(1),
        }
    }
}

impl HostInputHandler for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<InputHandler>> {
        let handler = InputHandler::new();
        let table_res = self.table.push(handler)?;
        let res = InputHandler::from_res(&table_res, &self.table)?;
        Ok(res)
    }

    fn handle_input(
        &mut self,
        _self_: Resource<InputHandler>,
    ) -> wasm_bridge::Result<Option<InputEvent>> {
        Ok(None)
    }

    fn drop(&mut self, rep: Resource<InputHandler>) -> wasm_bridge::Result<()> {
        InputHandler::handle_drop(rep, &mut self.table)
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_drop() {
        let (mut state, _) = StoreState::new("test_drop".to_string());

        let res = HostInputHandler::new(&mut state).unwrap();

        crate::api::utils::tests::test_drop(&mut state, res);
    }
}
