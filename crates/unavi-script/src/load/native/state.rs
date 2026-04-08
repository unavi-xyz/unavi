use std::sync::{Arc, Mutex};

use bevy::prelude::Entity;
use bevy_hsd::{cache::SceneRegistryInner, hydrate::events::ScriptCommandQueue};
use loro::LoroDoc;
use smol_str::SmolStr;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

use crate::{
    event_registry::EventRegistry,
    input_registry::InputRegistry,
    native::{
        agent::ProxyRegistry,
        api::wired::{
            agent::WiredAgentRt,
            event::WiredEventRt,
            input::WiredInputRt,
            scene::{GlobalRegistryMap, WiredSceneRt},
            wds::WiredWdsRt,
        },
    },
};

pub struct StoreState {
    wasi: WasiCtx,
    resource_table: ResourceTable,
    pub rt: RuntimeData,
}

pub struct RuntimeData {
    pub wired_agent: WiredAgentRt,
    pub wired_event: WiredEventRt,
    pub wired_input: WiredInputRt,
    pub wired_scene: WiredSceneRt,
    pub wired_wds: WiredWdsRt,
}

impl RuntimeData {
    #[must_use]
    pub fn new(
        actor: Option<wds::actor::Actor>,
        blobs: Option<wds::Blobs>,
        doc: Arc<LoroDoc>,
        self_node_id: SmolStr,
        registry: Arc<SceneRegistryInner>,
        command_queue: Arc<Mutex<ScriptCommandQueue>>,
        agent_entry: Option<Arc<ProxyRegistry>>,
        doc_id: blake3::Hash,
        doc_entity: Entity,
        input_registry: InputRegistry,
        event_registry: EventRegistry,
        registry_map: GlobalRegistryMap,
    ) -> Self {
        Self {
            wired_agent: WiredAgentRt {
                local_agent: agent_entry,
                table: ResourceTable::default(),
            },
            wired_event: WiredEventRt {
                registry: event_registry,
                table: ResourceTable::default(),
            },
            wired_input: WiredInputRt {
                registry: input_registry,
                table: ResourceTable::default(),
            },
            wired_scene: WiredSceneRt {
                actor: actor.clone(),
                blobs,
                doc,
                doc_entity,
                doc_id,
                self_node_id,
                table: ResourceTable::default(),
                registry,
                command_queue,
                registry_map,
            },
            wired_wds: WiredWdsRt {
                actor,
                table: ResourceTable::default(),
            },
        }
    }
}

impl WasiView for StoreState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.resource_table,
        }
    }
}

impl StoreState {
    #[must_use]
    pub fn new(wasi: WasiCtx, rt: RuntimeData) -> Self {
        Self {
            wasi,
            resource_table: ResourceTable::default(),
            rt,
        }
    }
}
