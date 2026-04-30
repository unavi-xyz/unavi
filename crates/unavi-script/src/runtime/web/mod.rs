use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

mod wired;

#[wasm_bindgen(module = "/dist/runtime.js")]
unsafe extern "C" {
    pub async fn build_script(bytes: &[u8], name: &str, runtime: Runtime);
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct WebRuntime {
    pub wired_agent: wired::agent::WiredAgent,
    pub wired_agent_types: wired::agent::WiredAgentTypes,
    pub wired_event: wired::event::WiredEvent,
    pub wired_event_types: wired::event::WiredEventTypes,
    pub wired_input: wired::input::WiredInput,
    pub wired_input_context: wired::input::WiredInputContext,
    pub wired_input_types: wired::input::WiredInputTypes,
    pub wired_portal: wired::portal::WiredPortal,
    pub wired_portal_types: wired::portal::WiredPortalTypes,
    pub wired_scene: wired::scene::WiredScene,
    pub wired_scene_types: wired::scene::WiredSceneTypes,
    pub wired_wds: wired::wds::WiredWds,
    pub wired_wds_types: wired::wds::WiredWdsTypes,
}
