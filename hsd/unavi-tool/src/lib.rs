//! A shell-hosted tool belt, shared by every tool crate and the shell that
//! holds them. `tool` is the tool side; `tool-registry` is the shell's.

mod protocol;
mod registry;
mod tool;

wired_guest::generate!();

struct World;

impl exports::unavi::tool::api::Guest for World {
    type Tool = tool::Tool;
    type ToolRegistry = registry::ToolRegistry;
}

export!(World);
