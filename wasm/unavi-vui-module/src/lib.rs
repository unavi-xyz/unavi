mod api;
mod discovery;
mod protocol;

wired_prelude::generate!();

struct World;

impl exports::unavi::vui_module::api::Guest for World {
    type VuiModule = api::VuiModuleImpl;
}

impl exports::unavi::vui_module::discovery::Guest for World {
    type ModuleDiscovery = discovery::ModuleDiscoveryImpl;
}

export!(World);
