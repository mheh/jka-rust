// The R4a 2D uber-shader v0: a textured quad tinted by its vertex colour.
//
// Vertex positions arrive in Raven's 640x480 virtual screen space (see
// `RB_SetGL2D`, oracle/codemp/renderer/tr_backend.cpp:1266-1292) and are
// mapped to clip space by the ortho matrix in group 0. The surface viewport
// does the virtual -> real resolution scaling, exactly as the oracle's
// `qglViewport(0, 0, glConfig.vidWidth, glConfig.vidHeight)` +
// `qglOrtho(0, 640, 480, 0, 0, 1)` pair did.

struct Transform2d {
    ortho: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> transform: Transform2d;

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = transform.ortho * vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, input.uv) * input.color;
}
