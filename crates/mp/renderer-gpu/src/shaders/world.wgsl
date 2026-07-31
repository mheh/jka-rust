// The R4 world uber-shader v0: a lightmapped BSP surface.
//
// Vertex positions arrive in world space (Raven's `drawVert_t`/`FaceVertex`
// xyz) and are mapped to clip space by the one clip matrix in group 0. That
// matrix is `correction * projection * modelMatrix`, where `modelMatrix` is the
// view orientation `R_RotateForViewer` builds and `correction` remaps GL's
// -1..1 clip z to wgpu's 0..1 (see `pipeline3d::world_clip_matrix`).
//
// The fragment output for this wave is `diffuse.rgb * lightmap.rgb` on a
// lightmapped surface, or `diffuse.rgb * vertex_color.rgb` where no lightmap
// exists. `SurfaceFlags.has_lightmap` selects between the two. tcMod, animMap,
// rgbGen waves, fog, and multi-stage shaders are out of this wave.

struct WorldGlobals {
    clip: mat4x4<f32>,
}

struct SurfaceFlags {
    has_lightmap: u32,
}

@group(0) @binding(0) var<uniform> globals: WorldGlobals;

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;
@group(1) @binding(2) var t_lightmap: texture_2d<f32>;
@group(1) @binding(3) var s_lightmap: sampler;

@group(2) @binding(0) var<uniform> surface: SurfaceFlags;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) st: vec2<f32>,
    @location(2) lightmap_st: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) st: vec2<f32>,
    @location(1) lightmap_st: vec2<f32>,
    @location(2) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = globals.clip * vec4<f32>(input.position, 1.0);
    output.st = input.st;
    output.lightmap_st = input.lightmap_st;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let diffuse = textureSample(t_diffuse, s_diffuse, input.st);
    if (surface.has_lightmap != 0u) {
        let lightmap = textureSample(t_lightmap, s_lightmap, input.lightmap_st);
        return vec4<f32>(diffuse.rgb * lightmap.rgb, diffuse.a);
    }
    return vec4<f32>(diffuse.rgb * input.color.rgb, diffuse.a);
}
