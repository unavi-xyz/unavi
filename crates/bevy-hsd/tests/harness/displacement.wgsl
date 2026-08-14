struct Params { inputs: array<vec4<f32>, 16> }
struct Globals { time: f32 }

@group(0) @binding(0) var<uniform> params: Params;
@group(1) @binding(0) var<uniform> globals: Globals;

//#HELPERS

@fragment
fn fragment() -> @location(0) vec4<f32> {
    var vertex: vec3<f32>;
    let graph_instance_index = 0u;
    let world_from_local = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
//#BODY
    return vec4<f32>(out_position_offset, 1.0)
        + vec4<f32>(out_normal_override, 1.0)
        + vec4<f32>(out_world_position_offset, 1.0);
}
