//! Parses and validates the portal shader with `naga`, so a malformed one
//! fails a `cargo test` rather than a run in a headset.
//!
//! Bevy's preprocessor is not WGSL: it resolves `#import` and substitutes
//! `#{...}` before wgpu sees the source, so both are stood in for here.

use naga::{
    front::wgsl,
    valid::{
        Capabilities,
        ValidationFlags,
        Validator,
    },
};

/// Declares only what the shader reads, which is also an assertion: a field
/// added to these stubs is a new demand on the mesh pipeline.
const STUBS: &str = "\
struct Vertex {
    @location(0) position: vec3<f32>,
}
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}
struct View {
    clip_from_world: mat4x4<f32>,
    world_from_clip: mat4x4<f32>,
    world_position: vec3<f32>,
    viewport: vec4<f32>,
}
struct Globals {
    time: f32,
}
@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(11) var<uniform> globals: Globals;
";

const MATERIAL_BIND_GROUP: &str = "3";

fn preprocess(source: &str) -> String {
    let mut body = String::new();
    let mut in_import = false;
    for line in source.lines() {
        if line.trim_start().starts_with("#import") {
            in_import = line.trim_end().ends_with('{');
            continue;
        }
        if in_import {
            in_import = line.trim() != "}";
            continue;
        }
        body.push_str(line);
        body.push('\n');
    }
    format!("{STUBS}{body}").replace("#{MATERIAL_BIND_GROUP}", MATERIAL_BIND_GROUP)
}

#[test]
fn the_portal_shader_is_valid_wgsl() {
    let source = preprocess(include_str!("../assets/portal.wgsl"));
    let module = match wgsl::parse_str(&source) {
        Ok(module) => module,
        Err(err) => panic!("{}", err.emit_to_string(&source)),
    };
    if let Err(err) = Validator::new(ValidationFlags::all(), Capabilities::all()).validate(&module)
    {
        panic!("{}", err.emit_to_string(&source));
    }
}
