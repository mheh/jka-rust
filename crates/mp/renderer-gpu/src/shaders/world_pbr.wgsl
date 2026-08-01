// The R4 PBR world uber-shader, v1 (DEC-37 ruling 5, backend #2).
//
// This backend shares the world vertex path and the group 0 / group 2 layout
// with the faithful `world.wgsl`. Group 1 grows to the PBR material set: the
// authored diffuse and the lightmap in bindings 0-3, plus the derived normal and
// roughness in bindings 4-7 (D21). A diffuse with no sidecar binds the shared
// neutral normal and roughness, which decode to a flat surface and mid
// roughness, so the lit result reduces to the plain irradiance.
//
// Routing (deliverable 3): `SurfaceFlags.pbr_lit` is 1 only for an opaque or
// lightmapped world or entity stage. An additive or blended effect stage (glow,
// sprite, blood, sky) carries 0, so this shader returns the faithful colour for
// it unchanged. The faithful base colour math matches `world.wgsl` exactly, so
// an effect stage looks the same in either backend.
//
// The lighting is normal-perturbed. The geometric normal comes from the
// screen-space derivatives of the surface position, and a cotangent frame built
// from the same derivatives maps the sampled normal map into that space (no
// vertex format change). One fixed light direction drives a diffuse-detail
// modulation of the baked irradiance plus a roughness-driven specular highlight.
// A world surface reads its irradiance from the lightmap. An entity reads it
// from the per-vertex CPU diffuse lighting colour. v1 proves the chain. The
// material tool slice replaces the derived data with authored data later.

// The light direction, in the space of the interpolated surface position. A
// world surface position is world space, so this is a fixed world sun. An entity
// position is model space, so the light rides the model. v1 accepts that, since
// the detail effect stays plausible either way.
const SUN_DIR: vec3<f32> = vec3<f32>(0.4, 0.6, 0.7);

// How far the perturbed normal modulates the baked irradiance. The modulation
// centers on 1.0, so it adds and removes light around the lightmap value rather
// than replacing it. This keeps the authored lighting and only adds bump detail.
const LIGHT_DETAIL: f32 = 0.5;

// The peak strength of the specular highlight at zero roughness. The highlight
// scales down with roughness and with the surface irradiance, so a dark surface
// never gains a bright sheen.
const SPEC_STRENGTH: f32 = 0.30;

// The Blinn-Phong exponent band. A smooth surface (low roughness) takes the high
// exponent for a tight highlight. A rough surface takes the low exponent for a
// wide, soft one.
const MIN_SHININESS: f32 = 6.0;
const MAX_SHININESS: f32 = 80.0;

struct WorldGlobals {
    clip: mat4x4<f32>,
}

struct SurfaceFlags {
    mode: u32,
    tex_from_lightmap: u32,
    alpha_func: u32,
    pbr_lit: u32,
}

@group(0) @binding(0) var<uniform> globals: WorldGlobals;

@group(1) @binding(0) var t_diffuse: texture_2d<f32>;
@group(1) @binding(1) var s_diffuse: sampler;
@group(1) @binding(2) var t_lightmap: texture_2d<f32>;
@group(1) @binding(3) var s_lightmap: sampler;
@group(1) @binding(4) var t_normal: texture_2d<f32>;
@group(1) @binding(5) var s_normal: sampler;
@group(1) @binding(6) var t_roughness: texture_2d<f32>;
@group(1) @binding(7) var s_roughness: sampler;

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
    // The surface position in its own space, so the fragment can derive the
    // geometric normal and the tangent frame from screen-space derivatives.
    @location(3) surface_pos: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = globals.clip * vec4<f32>(input.position, 1.0);
    output.st = input.st;
    output.lightmap_st = input.lightmap_st;
    output.color = input.color;
    output.surface_pos = input.position;
    return output;
}

// Perturbs the geometric normal by the sampled tangent-space normal, with a
// cotangent frame built from the position and texcoord derivatives. This is the
// standard tangent-free normal mapping trick. A degenerate uv (a stage that
// reads a constant texcoord) leaves the normal flat.
fn perturb_normal(n_geom: vec3<f32>, dp_x: vec3<f32>, dp_y: vec3<f32>, uv: vec2<f32>) -> vec3<f32> {
    let sample = textureSample(t_normal, s_normal, uv).xyz * 2.0 - 1.0;

    let duv_x = dpdx(uv);
    let duv_y = dpdy(uv);
    let denom = duv_x.x * duv_y.y - duv_y.x * duv_x.y;
    if (abs(denom) < 1e-8) {
        return n_geom;
    }

    // The raw tangent from the derivative solve, made orthogonal to the normal.
    let raw_t = (dp_x * duv_y.y - dp_y * duv_x.y) / denom;
    let ortho_t = raw_t - n_geom * dot(n_geom, raw_t);
    // A tangent parallel to the normal solves to a zero vector, and normalize of
    // a zero vector writes a NaN fragment. The flat normal is the safe fallback.
    if (length(ortho_t) < 1e-8) {
        return n_geom;
    }
    let t = normalize(ortho_t);
    // `n_geom` faces the viewer after the negation in `fs_main`, so `cross(t,
    // n_geom)` reproduces the pre-negation bitangent and keeps the green channel.
    let b = normalize(cross(t, n_geom));
    return normalize(sample.x * t + sample.y * b + sample.z * n_geom);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // The faithful base colour, identical to `world.wgsl`. `albedo` is the
    // material colour and `irradiance` is the light reaching the surface, so
    // `albedo * irradiance` reproduces the faithful output.
    var albedo: vec3<f32>;
    var irradiance: vec3<f32>;
    var alpha: f32;
    var uv = input.st;
    if (surface.mode == 1u) {
        let diffuse = textureSample(t_diffuse, s_diffuse, input.st);
        let lightmap = textureSample(t_lightmap, s_lightmap, input.lightmap_st);
        albedo = diffuse.rgb;
        irradiance = lightmap.rgb;
        alpha = diffuse.a;
    } else {
        if (surface.tex_from_lightmap != 0u) {
            uv = input.lightmap_st;
        }
        let tex = textureSample(t_diffuse, s_diffuse, uv);
        albedo = tex.rgb;
        irradiance = input.color.rgb;
        alpha = tex.a * input.color.a;
    }

    var rgb = albedo * irradiance;

    // The lit path perturbs the normal and adds detail plus specular. An effect
    // stage skips this and keeps the faithful colour.
    if (surface.pbr_lit == 1u) {
        let dp_x = dpdx(input.surface_pos);
        let dp_y = dpdy(input.surface_pos);
        // `cross(dp_x, dp_y)` faces away from the viewer, because the wgpu
        // framebuffer y points down while the NDC y points up. The negation
        // turns the outward normal toward the viewer and the sun. A
        // screen-degenerate quad solves the cross product to zero, so the length
        // test keeps `normalize` from writing a NaN fragment.
        let face = cross(dp_x, dp_y);
        if (length(face) > 1e-8) {
            let n_geom = -normalize(face);
            let n_pert = perturb_normal(n_geom, dp_x, dp_y, uv);

            let l = normalize(SUN_DIR);
            // The detail centers on 1.0: the bump adds or removes light around the
            // baked irradiance, so the authored lighting stays.
            let detail = 1.0 + LIGHT_DETAIL * (dot(n_pert, l) - dot(n_geom, l));

            // Roughness picks the highlight width and strength. The half vector uses
            // the geometric normal as a cheap stand-in for the view direction, which
            // needs no camera position and stays stable in any space.
            let rough = textureSample(t_roughness, s_roughness, vec2<f32>(0.5, 0.5)).r;
            let shininess = mix(MAX_SHININESS, MIN_SHININESS, rough);
            // A surface that faces straight away from the sun solves `l + n_geom`
            // to zero, so the length test keeps the highlight off it rather than
            // let `normalize` write a NaN fragment.
            let hv = l + n_geom;
            var spec = 0.0;
            if (length(hv) > 1e-8) {
                let h = normalize(hv);
                spec = pow(max(dot(n_pert, h), 0.0), shininess) * (1.0 - rough) * SPEC_STRENGTH;
            }

            // The specular ties to the surface irradiance, so a dark area gains no
            // sheen.
            rgb = rgb * max(detail, 0.0) + irradiance * spec;
        }
    }

    var color = vec4<f32>(rgb, alpha);

    // Alpha test discards a fragment the GLS_ATEST bits reject, before blending.
    // Codes: 0 none, 1 GT_0, 2 LT_80 (< 0.5), 3 GE_80 (>= 0.5), 4 GE_C0 (>= 0.75).
    if (surface.alpha_func == 1u && color.a <= 0.0) { discard; }
    if (surface.alpha_func == 2u && color.a >= 0.5) { discard; }
    if (surface.alpha_func == 3u && color.a < 0.5) { discard; }
    if (surface.alpha_func == 4u && color.a < 0.75) { discard; }

    return color;
}
