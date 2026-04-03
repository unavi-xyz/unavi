mod protocol;
mod registry;
mod vui_module;

wired_prelude::generate!();

struct World;

impl exports::unavi::vui_module::api::Guest for World {
    type VuiModule = vui_module::VuiModule;
    type VuiModuleRegistry = registry::VuiModuleRegistry;
}

export!(World);
