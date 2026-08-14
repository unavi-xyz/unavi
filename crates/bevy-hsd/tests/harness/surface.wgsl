struct In { uv: vec2<f32>, world_position: vec4<f32>, world_normal: vec3<f32>, color: vec4<f32>, instance_index: u32 }
struct PbrInput { world_normal: vec3<f32> }
struct Params { inputs: array<vec4<f32>, 16> }
struct Globals { time: f32 }

@group(0) @binding(0) var<uniform> params: Params;
@group(1) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(1) var tex_0: texture_2d<f32>;
@group(1) @binding(2) var samp_0: sampler;
@group(1) @binding(3) var tex_1: texture_2d<f32>;
@group(1) @binding(4) var samp_1: sampler;
@group(1) @binding(5) var tex_2: texture_2d<f32>;
@group(1) @binding(6) var samp_2: sampler;
@group(1) @binding(7) var tex_3: texture_2d<f32>;
@group(1) @binding(8) var samp_3: sampler;

//#HELPERS

@fragment
fn fragment(in: In) -> @location(0) vec4<f32> {
    var pbr_input: PbrInput;
    let N = in.world_normal;
    let V = in.world_normal;
    let graph_instance_index = in.instance_index;
    let world_from_local = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
//#BODY
    return {OUT_EXPR};
}
