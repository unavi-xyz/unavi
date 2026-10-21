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
use unavi_portal::view::still;

/// Declares only what the shader reads, which is also an assertion: a field
/// added to these stubs is a new demand on the mesh pipeline.
const PORTAL_STUBS: &str = "\
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

const DISTANCE_STUBS: &str = "\
struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
";

const MATERIAL_BIND_GROUP: &str = "3";

fn preprocess(stubs: &str, source: &str) -> String {
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
    format!("{stubs}{body}").replace("#{MATERIAL_BIND_GROUP}", MATERIAL_BIND_GROUP)
}

fn assert_valid(source: &str) {
    let module = match wgsl::parse_str(source) {
        Ok(module) => module,
        Err(err) => panic!("{}", err.emit_to_string(source)),
    };
    if let Err(err) = Validator::new(ValidationFlags::all(), Capabilities::all()).validate(&module)
    {
        panic!("{}", err.emit_to_string(source));
    }
}

#[test]
fn the_portal_shader_is_valid_wgsl() {
    assert_valid(&preprocess(
        PORTAL_STUBS,
        include_str!("../assets/portal.wgsl"),
    ));
}

#[test]
fn the_still_distance_shader_is_valid_wgsl() {
    assert_valid(&preprocess(
        DISTANCE_STUBS,
        include_str!("../assets/still_distance.wgsl"),
    ));
}

#[test]
fn the_still_distance_shader_mirrors_the_capture_constants() {
    let source = include_str!("../assets/still_distance.wgsl");
    for constant in [
        format!("const FACE_SIZE: f32 = {:?};", still::FACE_SIZE as f32),
        format!("const NEAR: f32 = {:?};", still::NEAR),
    ] {
        assert!(
            source.contains(&constant),
            "`{constant}` is not in the shader"
        );
    }
}
