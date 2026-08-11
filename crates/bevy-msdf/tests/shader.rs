//! Parses the distance-field shader with `naga`, so a malformed one fails a
//! `cargo test` rather than a run in a headset.
//!
//! Bevy's preprocessor is not WGSL: it resolves `#import` and substitutes
//! `#{...}` before wgpu ever sees the source. Both are stood in for here, the
//! same way `bevy-hsd`'s codegen test does.

use naga::front::wgsl;

/// Declares only what the shader reads, which is also an assertion: a field
/// added to this stub is a new demand on the mesh pipeline.
const VERTEX_OUTPUT: &str = "\
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}
";

const MATERIAL_BIND_GROUP: &str = "3";

fn preprocess(source: &str) -> String {
    let body = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("#import"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{VERTEX_OUTPUT}{body}").replace("#{MATERIAL_BIND_GROUP}", MATERIAL_BIND_GROUP)
}

#[test]
fn the_distance_field_shader_is_valid_wgsl() {
    let source = preprocess(include_str!("../src/msdf.wgsl"));
    if let Err(err) = wgsl::parse_str(&source) {
        panic!("{}", err.emit_to_string(&source));
    }
}

#[test]
fn the_shader_binds_the_group_bevy_gives_a_material() {
    let source = include_str!("../src/msdf.wgsl");
    assert!(
        !source.contains("@group(3)"),
        "a hardcoded index silently breaks the frame Bevy renumbers its groups"
    );
    for binding in 0..3 {
        assert!(
            source.contains(&format!(
                "@group(#{{MATERIAL_BIND_GROUP}}) @binding({binding})"
            )),
            "binding {binding} must match the AsBindGroup attribute order"
        );
    }
}
