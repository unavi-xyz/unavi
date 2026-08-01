mod protocol;
mod registry;
mod tool;

wired_prelude::generate!();

struct World;

impl exports::unavi::gauntlet_tool::api::Guest for World {
    type Tool = tool::Tool;
    type ToolRegistry = registry::ToolRegistry;
}

export!(World);
