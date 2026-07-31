// The R4 world uber-shader v0: a lightmapped BSP surface.
//
// Vertex positions arrive in world space (Raven's `drawVert_t`/`FaceVertex`
// xyz) and are mapped to clip space by one clip matrix in group 0. Group 0
// holds one 256-byte slot per distinct entity number the scene draws, and the
// draw selects its slot with a dynamic offset. That matrix is
// `correction * projection * modelMatrix`. For the world entity, `modelMatrix`
// is the view orientation `R_RotateForViewer` builds. For a real entity (an
// inline brush model), it is `R_RotateForEntity`'s output. `correction` remaps
// GL's -1..1 clip z to wgpu's 0..1 (see `pipeline3d::world_clip_matrix`).
//
// One pass draws one shader stage. `SurfaceFlags.mode` picks the path:
//   mode 0 (single texture): `texture(uv) * color`, the common stage pass.
//   mode 1 (two texture): `diffuse.rgb * lightmap.rgb`, the GL_MODULATE collapse.
// In mode 0, `tex_from_lightmap` reads the lightmap st for a lightmap stage;
// a dynamic stage instead writes its resolved st into the `st` field, so the
// flag stays 0. The per-vertex `color` carries the stage's rgbGen/alphaGen
// result (or the BSP vertex colour on a static stage).

struct WorldGlobals {
    clip: mat4x4<f32>,
}

struct SurfaceFlags {
    mode: u32,
    tex_from_lightmap: u32,
    alpha_func: u32,
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
    var color: vec4<f32>;
    if (surface.mode == 1u) {
        let diffuse = textureSample(t_diffuse, s_diffuse, input.st);
        let lightmap = textureSample(t_lightmap, s_lightmap, input.lightmap_st);
        color = vec4<f32>(diffuse.rgb * lightmap.rgb, diffuse.a);
    } else {
        var uv = input.st;
        if (surface.tex_from_lightmap != 0u) {
            uv = input.lightmap_st;
        }
        let tex = textureSample(t_diffuse, s_diffuse, uv);
        color = tex * input.color;
    }

    // Alpha test discards a fragment the GLS_ATEST bits reject, before blending.
    // Codes: 0 none, 1 GT_0, 2 LT_80 (< 0.5), 3 GE_80 (>= 0.5), 4 GE_C0 (>= 0.75).
    if (surface.alpha_func == 1u && color.a <= 0.0) { discard; }
    if (surface.alpha_func == 2u && color.a >= 0.5) { discard; }
    if (surface.alpha_func == 3u && color.a < 0.5) { discard; }
    if (surface.alpha_func == 4u && color.a < 0.75) { discard; }
    return color;
}
