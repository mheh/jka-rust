//! `stage2d` — one shader stage's colour, texture coordinates and bound image,
//! reduced to what a 2D stretch-pic needs (R4a backend #1, wave 4).
//!
//! `RB_StageIteratorGeneric` draws one pass per active stage, and each pass is
//! three per-stage computations plus a state change: `ComputeColors` fills
//! `tess.svars.colors`, `ComputeTexCoords` fills `tess.svars.texcoords`,
//! `R_BindAnimatedImage` picks the bundle's texture, and `GL_State` installs
//! `pStage->stateBits`. This module is those three computations for the one
//! surface kind backend #1 rasterises — the four-vertex quad `RB_StretchPic`
//! builds, whose vertex colours are all `backEnd.color2D` (the `RE_SetColor`
//! register) and whose texture coordinates are the command's `s1/t1/s2/t2`.
//!
//! Everything here is the oracle's own arithmetic: the generator switches are
//! transcribed below, and every wave/tcMod evaluator is `mp_renderer`'s
//! already-ported `tr_shade_calc` function called with the 4-element arrays a
//! quad has instead of `tess`'s 1000.
//!
//! Source: `oracle/codemp/renderer/tr_shade.cpp:1529-1801` (`ComputeColors`),
//! `:1809-1927` (`ComputeTexCoords`), `:239-290` (`R_BindAnimatedImage`)

use std::collections::BTreeSet;

use mp_renderer::render_state::image_asset::ImageHandle;
use mp_renderer::render_state::placeholders::{FUNCTABLE_SIZE, FUNCTABLE_SIZE2};
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::render_state::shader_stage::ShaderStage;
use mp_renderer::render_state::texture_bundle::TextureBundle;
use mp_renderer::tr_local::alpha_gen_t::alphaGen_t;
use mp_renderer::tr_local::color_gen_t::colorGen_t;
use mp_renderer::tr_local::tex_coord_gen_t::texCoordGen_t;
use mp_renderer::tr_local::tex_mod_t::texMod_t;
use mp_renderer::tr_noise::NoiseState;
use mp_renderer::tr_shade_calc::{
    myftol, RB_CalcRotateTexCoords, RB_CalcScaleTexCoords, RB_CalcScrollTexCoords,
    RB_CalcStretchTexCoords, RB_CalcTransformTexCoords, RB_CalcWaveAlpha, RB_CalcWaveColor,
};

/// `tr.identityLight`.
///
/// DEC-37 ruling 10 makes overbright an output pass rather than a texture-time
/// shift; backend #1 has no such pass yet, so the 2D path runs at retail's
/// windowed-mode value (`tr.overbrightBits == 0` -> `1 / (1 << 0)`).
///
/// Source: `oracle/codemp/renderer/tr_image.cpp` (`R_SetColorMappings`)
pub const IDENTITY_LIGHT: f32 = 1.0;

/// `tr.identityLightByte` — `255 * tr.identityLight`.
const IDENTITY_LIGHT_BYTE: u8 = 255;

/// The four vertices `RB_StretchPic` emits per pic, in its own order: the
/// quad's top-left, top-right, bottom-right, bottom-left corners.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1461-1490`
pub const QUAD_VERTS: usize = 4;

/// The per-frame clock the generators read, in the shape `RB_SetGL2D` and
/// `RB_BeginSurface` leave it: `backEnd.refdef.time` is the 2D pass's
/// millisecond stamp, `floatTime` is that in seconds, and `shaderTime` is
/// `floatTime` minus the shader's own `timeOffset`.
///
/// Source: `oracle/codemp/renderer/tr_backend.cpp:1289-1291` (`RB_SetGL2D`);
/// `oracle/codemp/renderer/tr_shade.cpp:161` (`RB_BeginSurface`)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageTime {
    /// `backEnd.refdef.time`.
    pub refdef_time: i32,
    /// `backEnd.refdef.floatTime`.
    pub refdef_float_time: f32,
    /// `tess.shaderTime`.
    pub shader_time: f32,
}

impl StageTime {
    /// The clock for a 2D pass at `float_time` seconds drawing a shader whose
    /// `timeOffset` is `time_offset`.
    pub fn new(float_time: f32, time_offset: f32) -> StageTime {
        StageTime {
            refdef_time: (float_time * 1000.0) as i32,
            refdef_float_time: float_time,
            shader_time: float_time - time_offset,
        }
    }
}

/// Which generator families this 2D path leaves to a later slice, tracked so a
/// shader that uses one logs once per kind per process instead of once per
/// frame.
#[derive(Default)]
pub struct Stage2dWarnings {
    seen: BTreeSet<(&'static str, i32)>,
}

impl Stage2dWarnings {
    /// Logs `kind`/`value` once, naming the shader that first reached it.
    fn once(&mut self, kind: &'static str, value: i32, shader_name: &str) {
        if self.seen.insert((kind, value)) {
            eprintln!(
                "mp_renderer_gpu: stage2d has no 2D path for {kind} {value} \
                 (first seen in shader '{shader_name}') — using the SetColor register"
            );
        }
    }

    /// Logs a skipped `tcMod` kind once.
    fn tcmod_once(&mut self, kind: &'static str, value: i32, shader_name: &str) {
        if self.seen.insert((kind, value)) {
            eprintln!(
                "mp_renderer_gpu: stage2d skips {kind} {value} \
                 (first seen in shader '{shader_name}')"
            );
        }
    }
}

/// `ComputeColors` for a stretch-pic quad: the one RGBA every vertex of the
/// quad carries, as the 0..1 floats [`crate::pipeline2d`]'s vertex format wants.
///
/// `vertex_color` is the `RE_SetColor` register in float form; the oracle
/// stores it as `backEnd.color2D`, a `byte[4]` quantised on the way in
/// (`RB_SetColor`'s `cmd->color[i] * 255`), so this quantises it the same way
/// before any `CGEN_VERTEX`/`AGEN_VERTEX` arithmetic touches it.
///
/// The oracle's entity-driven arms (`CGEN_ENTITY`, `CGEN_LIGHTING_*`,
/// `AGEN_LIGHTING_SPECULAR`, …) read `backEnd.currentEntity` — for a 2D pic
/// that is `backEnd.entity2D`, a zeroed `trRefEntity_t`, which no menu shader
/// in the retail tree relies on. They fall back to the `RE_SetColor` register
/// with a once-per-kind log rather than being guessed at. `CGEN_BAD` keeps the
/// oracle's own `default:` arm (identity lighting).
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1591-1779`
#[allow(clippy::too_many_arguments)]
pub fn stage_color(
    stage: &ShaderStage,
    vertex_color: [f32; 4],
    time: StageTime,
    noise: &NoiseState,
    assets: &RenderAssets,
    shader_name: &str,
    warnings: &mut Stage2dWarnings,
) -> [f32; 4] {
    // `RB_SetColor`: `backEnd.color2D[i] = cmd->color[i] * 255` — a C
    // float->byte conversion, truncating toward zero.
    let vertex = [
        (vertex_color[0] * 255.0) as u8,
        (vertex_color[1] * 255.0) as u8,
        (vertex_color[2] * 255.0) as u8,
        (vertex_color[3] * 255.0) as u8,
    ];
    let mut colors = [vertex; QUAD_VERTS];

    // `forceRGBGen` is 0 (`CGEN_BAD`) for every 2D pass — `RB_StageIteratorGeneric`
    // passes it only for the entity-forced paths — so it always falls back to
    // the stage's own `rgbGen`.
    let force_rgb_gen = stage.rgb_gen;

    //
    // rgbGen
    //
    match force_rgb_gen {
        colorGen_t::CGEN_IDENTITY => colors = [[0xff; 4]; QUAD_VERTS],
        colorGen_t::CGEN_IDENTITY_LIGHTING | colorGen_t::CGEN_BAD => {
            colors = [[IDENTITY_LIGHT_BYTE; 4]; QUAD_VERTS]
        }
        colorGen_t::CGEN_EXACT_VERTEX => colors = [vertex; QUAD_VERTS],
        colorGen_t::CGEN_CONST => colors = [stage.constant_color; QUAD_VERTS],
        colorGen_t::CGEN_VERTEX => {
            if IDENTITY_LIGHT == 1.0 {
                colors = [vertex; QUAD_VERTS];
            } else {
                for c in colors.iter_mut() {
                    c[0] = (vertex[0] as f32 * IDENTITY_LIGHT) as u8;
                    c[1] = (vertex[1] as f32 * IDENTITY_LIGHT) as u8;
                    c[2] = (vertex[2] as f32 * IDENTITY_LIGHT) as u8;
                    c[3] = vertex[3];
                }
            }
        }
        colorGen_t::CGEN_ONE_MINUS_VERTEX => {
            for c in colors.iter_mut() {
                if IDENTITY_LIGHT == 1.0 {
                    c[0] = 255 - vertex[0];
                    c[1] = 255 - vertex[1];
                    c[2] = 255 - vertex[2];
                } else {
                    c[0] = ((255 - vertex[0]) as f32 * IDENTITY_LIGHT) as u8;
                    c[1] = ((255 - vertex[1]) as f32 * IDENTITY_LIGHT) as u8;
                    c[2] = ((255 - vertex[2]) as f32 * IDENTITY_LIGHT) as u8;
                }
            }
        }
        colorGen_t::CGEN_WAVEFORM => RB_CalcWaveColor(
            &stage.rgb_wave,
            &mut colors,
            noise,
            time.refdef_time,
            time.refdef_float_time,
            time.shader_time,
            IDENTITY_LIGHT,
            assets,
            shader_name,
        ),
        other => warnings.once("rgbGen", other as i32, shader_name),
    }

    //
    // alphaGen
    //
    match stage.alpha_gen {
        alphaGen_t::AGEN_SKIP => {}
        alphaGen_t::AGEN_IDENTITY => {
            if force_rgb_gen != colorGen_t::CGEN_IDENTITY
                && ((force_rgb_gen == colorGen_t::CGEN_VERTEX && IDENTITY_LIGHT != 1.0)
                    || force_rgb_gen != colorGen_t::CGEN_VERTEX)
            {
                for c in colors.iter_mut() {
                    c[3] = 0xff;
                }
            }
        }
        alphaGen_t::AGEN_CONST => {
            if force_rgb_gen != colorGen_t::CGEN_CONST {
                for c in colors.iter_mut() {
                    c[3] = stage.constant_color[3];
                }
            }
        }
        alphaGen_t::AGEN_WAVEFORM => RB_CalcWaveAlpha(
            &stage.alpha_wave,
            &mut colors,
            noise,
            time.refdef_time,
            time.refdef_float_time,
            time.shader_time,
            assets,
            shader_name,
        ),
        alphaGen_t::AGEN_VERTEX => {
            if force_rgb_gen != colorGen_t::CGEN_VERTEX {
                for c in colors.iter_mut() {
                    c[3] = vertex[3];
                }
            }
        }
        alphaGen_t::AGEN_ONE_MINUS_VERTEX => {
            for c in colors.iter_mut() {
                c[3] = 255 - vertex[3];
            }
        }
        other => warnings.once("alphaGen", other as i32, shader_name),
    }

    // Every vertex of a stretch-pic quad carries the same colour, so the whole
    // `svars.colors` run collapses to element 0.
    [
        colors[0][0] as f32 / 255.0,
        colors[0][1] as f32 / 255.0,
        colors[0][2] as f32 / 255.0,
        colors[0][3] as f32 / 255.0,
    ]
}

/// `ComputeTexCoords` for bundle 0 of a stretch-pic quad: the command's own
/// `s1/t1/s2/t2` corners run through the bundle's `tcMod` list.
///
/// `st` arrives as the four corners in `RB_StretchPic`'s vertex order and is
/// rewritten in place, so a rotate — which is not expressible as a
/// [`crate::pipeline2d::UvRect`] — comes out as four independent corners.
///
/// `TCGEN_TEXTURE` is what `FinishShader` defaults every non-lightmap stage to
/// and is the identity here (the corners already *are* `tess.texCoords[i][0]`).
/// `TCGEN_IDENTITY` zeroes them, as the oracle does. The world-geometry gens
/// (lightmap, vector, fog, environment) have no 2D meaning and are logged once
/// and left alone. `TMOD_TURBULENT`/`TMOD_ENTITY_TRANSLATE` read `tess.xyz` /
/// `backEnd.currentEntity` and are logged once and skipped.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:1814-1925`
#[allow(clippy::too_many_arguments)]
pub fn stage_texcoords(
    bundle: &TextureBundle,
    st: &mut [[f32; 2]; QUAD_VERTS],
    time: StageTime,
    noise: &NoiseState,
    assets: &RenderAssets,
    shader_name: &str,
    warnings: &mut Stage2dWarnings,
) {
    //
    // generate the texture coordinates
    //
    match bundle.tc_gen {
        texCoordGen_t::TCGEN_IDENTITY => *st = [[0.0; 2]; QUAD_VERTS],
        texCoordGen_t::TCGEN_TEXTURE => {}
        texCoordGen_t::TCGEN_BAD => return,
        other => warnings.once("tcGen", other as i32, shader_name),
    }

    //
    // alter texture coordinates
    //
    for tmi in &bundle.tex_mods {
        match tmi.r#type {
            // break out of for loop
            texMod_t::TMOD_NONE => break,
            texMod_t::TMOD_SCROLL => {
                // scroll unioned
                RB_CalcScrollTexCoords(tmi.translate, st, time.shader_time)
            }
            texMod_t::TMOD_SCALE => RB_CalcScaleTexCoords(tmi.translate, st),
            texMod_t::TMOD_STRETCH => RB_CalcStretchTexCoords(
                &tmi.wave,
                st,
                noise,
                time.refdef_time,
                time.refdef_float_time,
                time.shader_time,
                assets,
                shader_name,
            ),
            texMod_t::TMOD_TRANSFORM => RB_CalcTransformTexCoords(tmi, st),
            texMod_t::TMOD_ROTATE => {
                RB_CalcRotateTexCoords(tmi.translate[0], st, time.shader_time, assets)
            }
            other => warnings.tcmod_once("tcMod", other as i32, shader_name),
        }
    }
}

/// `R_BindAnimatedImage`'s frame selection: the texture a bundle binds at
/// `shader_time`.
///
/// `None` means "bind nothing" — either the bundle is a `videoMap` (the
/// cinematic path, which uploads into `tr.scratchImage` and never touches
/// `bundle->image`) or it never got an image at all.
///
/// The lightmap/`RF_SETANIMINDEX` arms have no 2D pic to reach them:
/// `r_fullbright` gates a lightmap bundle, and `RF_SETANIMINDEX` reads the
/// current entity, which is the zeroed `backEnd.entity2D` here.
///
/// Source: `oracle/codemp/renderer/tr_shade.cpp:239-290`
pub fn stage_image(bundle: &TextureBundle, shader_time: f32) -> Option<ImageHandle> {
    if bundle.is_video_map {
        //TODO: Port CIN_RunCinematic/CIN_UploadCinematic
        // Source: oracle/codemp/renderer/tr_shade.cpp:243-244
        // Scoped follow-up, not a gap in this file: the whole ROQ decoder
        // (`oracle/codemp/client/cl_cin.cpp`, 1,494 lines / 38 fns) plus the
        // 10-fn `CIN_*` surface (`oracle/codemp/client/client.h:577-590`), the
        // 5-slot `UI_CIN_*`/`CG_CIN_*` trap families
        // (`oracle/codemp/ui/ui_public.h:105-109`,
        // `oracle/codemp/cgame/cg_public.h:210-214`) and a `RenderAssets` home
        // for `tr.scratchImage[16]` — which `R_CreateBuiltinImages` already
        // creates and drops (see `tr_image.rs`'s ESCALATION there).
        // `ParseStage`'s `videoMap` arm never sets `bundle->image` without a
        // live cinematic handle, so `FinishShader` deactivates the stage and
        // the shader draws no pass at all — binding nothing here is that same
        // outcome, not a fallback.
        return None;
    }

    if bundle.num_image_animations <= 1 {
        return bundle.image;
    }

    // it is necessary to do this messy calc to make sure animations line up
    // exactly with waveforms of the same frequency
    let mut index = myftol(shader_time * bundle.image_animation_speed * FUNCTABLE_SIZE as f32);
    index >>= FUNCTABLE_SIZE2;

    if index < 0 {
        index = 0; // may happen with shader time offsets
    }

    if bundle.one_shot_anim_map {
        if index >= bundle.num_image_animations as i32 {
            // stick on last frame
            index = bundle.num_image_animations as i32 - 1;
        }
    } else {
        // loop
        index %= bundle.num_image_animations as i32;
    }

    bundle.image_animations.get(index as usize).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mp_renderer::render_state::placeholders::FunctionTables;
    use mp_renderer::tr_local::acff_t::acff_t;
    use mp_renderer::tr_local::eglfog_override::EGLFogOverride;
    use mp_renderer::tr_local::gen_func_t::genFunc_t;
    use mp_renderer::tr_local::tex_mod_info_t::texModInfo_t;
    use mp_renderer::tr_local::wave_form_t::waveForm_t;
    use mp_renderer::tr_shader::NUM_TEXTURE_BUNDLES;
    use std::f64::consts::PI;

    use crate::ui_host::boot::empty_assets;

    /// `R_InitFuncTables`' `sinTable` fill — the one table the `tcMod rotate`
    /// path reads.
    ///
    /// Source: `oracle/codemp/renderer/tr_init.cpp:1256-1259`
    fn test_assets() -> RenderAssets {
        let mut assets = empty_assets();
        let mut tables = FunctionTables::default();
        for (i, entry) in tables.sin_table.iter_mut().enumerate() {
            let deg = i as f32 * 360.0 / (FUNCTABLE_SIZE - 1) as f32;
            *entry = (deg as f64 * PI / 180.0).sin() as f32;
        }
        assets.function_tables = tables;
        assets
    }

    fn flat_wave() -> waveForm_t {
        waveForm_t {
            func: genFunc_t::GF_NONE,
            base: 0.0,
            amplitude: 0.0,
            phase: 0.0,
            frequency: 0.0,
        }
    }

    /// The zeroed `textureBundle_t` `R_FindShader`'s `Com_Memset(&stages, …)`
    /// leaves behind, before any `map`/`animMap` keyword fills it in.
    fn empty_bundle() -> TextureBundle {
        TextureBundle {
            image: None,
            tc_gen: texCoordGen_t::TCGEN_BAD,
            tc_gen_vectors: [[0.0; 3]; 2],
            tex_mods: Vec::new(),
            num_image_animations: 0,
            image_animation_speed: 0.0,
            is_lightmap: false,
            one_shot_anim_map: false,
            vertex_lightmap: false,
            is_video_map: false,
            video_map_handle: 0,
            image_animations: Vec::new(),
        }
    }

    fn stage(rgb_gen: colorGen_t, alpha_gen: alphaGen_t) -> ShaderStage {
        ShaderStage {
            active: true,
            is_detail: false,
            index: 0,
            lightmap_style: 0,
            bundle: std::array::from_fn(|_| empty_bundle()),
            rgb_wave: flat_wave(),
            rgb_gen,
            alpha_wave: flat_wave(),
            alpha_gen,
            constant_color: [10, 20, 30, 40],
            state_bits: 0,
            adjust_colors_for_fog: acff_t::ACFF_NONE,
            gl_fog_color_override: EGLFogOverride::GLFOGOVERRIDE_NONE,
            ss: None,
            glow: false,
        }
    }

    /// The `RE_SetColor` register the harness's menus paint with, mid-scale so
    /// the vertex arms are distinguishable from the identity ones.
    const REGISTER: [f32; 4] = [0.5, 0.25, 0.75, 0.5];

    fn color(rgb_gen: colorGen_t, alpha_gen: alphaGen_t) -> [f32; 4] {
        let assets = test_assets();
        let noise = NoiseState::default();
        let mut warnings = Stage2dWarnings::default();
        stage_color(
            &stage(rgb_gen, alpha_gen),
            REGISTER,
            StageTime::new(0.0, 0.0),
            &noise,
            &assets,
            "test",
            &mut warnings,
        )
    }

    /// `backEnd.color2D` quantised back to floats — what every "use the
    /// register" arm must produce.
    fn quantised_register() -> [f32; 4] {
        [
            (REGISTER[0] * 255.0) as u8 as f32 / 255.0,
            (REGISTER[1] * 255.0) as u8 as f32 / 255.0,
            (REGISTER[2] * 255.0) as u8 as f32 / 255.0,
            (REGISTER[3] * 255.0) as u8 as f32 / 255.0,
        ]
    }

    #[test]
    fn identity_gens_are_white() {
        // `CGEN_IDENTITY` memsets all four bytes to 0xff, so `AGEN_IDENTITY`'s
        // guard correctly skips its own store.
        assert_eq!(
            color(colorGen_t::CGEN_IDENTITY, alphaGen_t::AGEN_IDENTITY),
            [1.0; 4]
        );
        // identityLight is 1 here, so identity lighting is the same white.
        assert_eq!(
            color(
                colorGen_t::CGEN_IDENTITY_LIGHTING,
                alphaGen_t::AGEN_IDENTITY
            ),
            [1.0; 4]
        );
        // `CGEN_BAD` takes the oracle's `default:` arm — identity lighting.
        assert_eq!(
            color(colorGen_t::CGEN_BAD, alphaGen_t::AGEN_IDENTITY),
            [1.0; 4]
        );
    }

    #[test]
    fn vertex_gens_take_the_setcolor_register() {
        let expected = quantised_register();
        // `AGEN_IDENTITY` after `CGEN_VERTEX` is skipped when identityLight is
        // 1 — the vertex alpha survives.
        assert_eq!(
            color(colorGen_t::CGEN_VERTEX, alphaGen_t::AGEN_IDENTITY),
            expected
        );
        assert_eq!(
            color(colorGen_t::CGEN_EXACT_VERTEX, alphaGen_t::AGEN_VERTEX),
            expected
        );
        // `AGEN_VERTEX` after a non-vertex rgbGen still writes the register's
        // alpha over the identity white.
        assert_eq!(
            color(colorGen_t::CGEN_IDENTITY, alphaGen_t::AGEN_VERTEX),
            [1.0, 1.0, 1.0, expected[3]],
            "rgb identity, alpha from the register"
        );
    }

    #[test]
    fn const_gens_take_the_stages_constant_color() {
        let constant = [10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0, 40.0 / 255.0];
        assert_eq!(
            color(colorGen_t::CGEN_CONST, alphaGen_t::AGEN_CONST),
            constant
        );
        // `AGEN_CONST` under a non-const rgbGen overrides only alpha.
        assert_eq!(
            color(colorGen_t::CGEN_IDENTITY, alphaGen_t::AGEN_CONST),
            [1.0, 1.0, 1.0, constant[3]]
        );
    }

    #[test]
    fn agen_skip_leaves_the_rgbgens_alpha_alone() {
        // `CGEN_CONST` wrote all four bytes; `AGEN_SKIP` must not touch alpha.
        assert_eq!(
            color(colorGen_t::CGEN_CONST, alphaGen_t::AGEN_SKIP)[3],
            40.0 / 255.0
        );
        // `CGEN_VERTEX` + `AGEN_SKIP` keeps the register's alpha.
        assert_eq!(
            color(colorGen_t::CGEN_VERTEX, alphaGen_t::AGEN_SKIP)[3],
            quantised_register()[3]
        );
    }

    #[test]
    fn one_minus_vertex_inverts_the_register() {
        let register = quantised_register();
        let inverted = color(
            colorGen_t::CGEN_ONE_MINUS_VERTEX,
            alphaGen_t::AGEN_ONE_MINUS_VERTEX,
        );
        for i in 0..4 {
            assert!(
                (inverted[i] - (1.0 - register[i])).abs() < 1.5 / 255.0,
                "component {i}: {} vs {}",
                inverted[i],
                1.0 - register[i]
            );
        }
    }

    #[test]
    fn an_unimplemented_gen_falls_back_to_the_register() {
        assert_eq!(
            color(colorGen_t::CGEN_LIGHTING_DIFFUSE, alphaGen_t::AGEN_SKIP),
            quantised_register()
        );
    }

    #[test]
    fn a_wave_rgbgen_greys_the_quad_by_the_waveform() {
        // `cursor`'s second stage: `rgbGen wave sin 0.5 0.35 0 0.6`. At t = 0
        // the sin table's phase-0 entry is 0, so the glow is the wave's base.
        let assets = test_assets();
        let noise = NoiseState::default();
        let mut warnings = Stage2dWarnings::default();
        let mut waved = stage(colorGen_t::CGEN_WAVEFORM, alphaGen_t::AGEN_IDENTITY);
        waved.rgb_wave = waveForm_t {
            func: genFunc_t::GF_SIN,
            base: 0.5,
            amplitude: 0.35,
            phase: 0.0,
            frequency: 0.6,
        };
        let color = stage_color(
            &waved,
            REGISTER,
            StageTime::new(0.0, 0.0),
            &noise,
            &assets,
            "cursor",
            &mut warnings,
        );
        assert!((color[0] - 0.5).abs() < 1.0 / 255.0, "{color:?}");
        assert_eq!(color[0], color[1]);
        assert_eq!(color[1], color[2]);
        assert_eq!(color[3], 1.0);
    }

    /// The unit-square corners in `RB_StretchPic`'s vertex order.
    fn unit_corners() -> [[f32; 2]; QUAD_VERTS] {
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    }

    fn bundle_with(tex_mods: Vec<texModInfo_t>) -> TextureBundle {
        TextureBundle {
            tc_gen: texCoordGen_t::TCGEN_TEXTURE,
            tex_mods,
            ..empty_bundle()
        }
    }

    fn texcoords(bundle: &TextureBundle, float_time: f32) -> [[f32; 2]; QUAD_VERTS] {
        let assets = test_assets();
        let noise = NoiseState::default();
        let mut warnings = Stage2dWarnings::default();
        let mut st = unit_corners();
        stage_texcoords(
            bundle,
            &mut st,
            StageTime::new(float_time, 0.0),
            &noise,
            &assets,
            "test",
            &mut warnings,
        );
        st
    }

    fn tex_mod(kind: texMod_t, translate: [f32; 2]) -> texModInfo_t {
        texModInfo_t {
            r#type: kind,
            wave: flat_wave(),
            matrix: [[0.0; 2]; 2],
            translate,
        }
    }

    #[test]
    fn tcmod_scroll_offsets_every_corner_by_speed_times_time() {
        // `gfx/menus/menu_side_text`'s `tcMod scroll 0 0.025` at t = 4s: 0.1 in
        // t, nothing in s, applied to all four corners equally.
        let bundle = bundle_with(vec![tex_mod(texMod_t::TMOD_SCROLL, [0.0, 0.025])]);
        let st = texcoords(&bundle, 4.0);
        for (moved, original) in st.iter().zip(unit_corners()) {
            assert!((moved[0] - original[0]).abs() < 1e-6);
            assert!(
                (moved[1] - (original[1] + 0.1)).abs() < 1e-6,
                "{moved:?} vs {original:?}"
            );
        }
        // The oracle clamps the accumulated scroll with `floor`, so a whole
        // number of periods later the corners are back where they started.
        let st = texcoords(&bundle, 40.0);
        for (moved, original) in st.iter().zip(unit_corners()) {
            assert!((moved[1] - original[1]).abs() < 1e-6, "{moved:?}");
        }
    }

    #[test]
    fn tcmod_rotate_turns_the_corners_about_the_texture_centre() {
        // `gfx/menus/main_ring`'s `tcMod rotate 5`.
        let bundle = bundle_with(vec![tex_mod(texMod_t::TMOD_ROTATE, [5.0, 0.0])]);
        // At t = 0 the rotation is the identity.
        for (turned, original) in texcoords(&bundle, 0.0).iter().zip(unit_corners()) {
            assert!((turned[0] - original[0]).abs() < 1e-5, "{turned:?}");
            assert!((turned[1] - original[1]).abs() < 1e-5, "{turned:?}");
        }
        // 18s at 5 deg/s is a quarter turn; the corners cycle round the square
        // and each stays its original distance from (0.5, 0.5).
        let turned = texcoords(&bundle, 18.0);
        assert_ne!(turned, unit_corners());
        for (turned, original) in turned.iter().zip(unit_corners()) {
            let before = (original[0] - 0.5f32).hypot(original[1] - 0.5);
            let after = (turned[0] - 0.5f32).hypot(turned[1] - 0.5);
            assert!(
                (before - after).abs() < 1e-4,
                "corner left the circle: {original:?} -> {turned:?}"
            );
        }
    }

    #[test]
    fn tcmod_none_stops_the_list() {
        let bundle = bundle_with(vec![
            tex_mod(texMod_t::TMOD_NONE, [0.0, 0.0]),
            tex_mod(texMod_t::TMOD_SCALE, [8.0, 8.0]),
        ]);
        assert_eq!(texcoords(&bundle, 1.0), unit_corners());
    }

    #[test]
    fn a_single_frame_bundle_binds_its_own_image() {
        let image = ImageHandle::new(5, 0);
        let bundle = TextureBundle {
            image: Some(image),
            ..empty_bundle()
        };
        assert!(stage_image(&bundle, 12.5) == Some(image));
    }

    #[test]
    fn an_animmap_bundle_loops_through_its_frames() {
        let frames: Vec<ImageHandle> = (0..3).map(|i| ImageHandle::new(i, 0)).collect();
        let bundle = TextureBundle {
            image: Some(frames[0]),
            num_image_animations: 3,
            // 4 fps: one frame per 0.25s.
            image_animation_speed: 4.0,
            image_animations: frames.clone(),
            ..empty_bundle()
        };
        assert!(stage_image(&bundle, 0.0) == Some(frames[0]));
        assert!(stage_image(&bundle, 0.3) == Some(frames[1]));
        assert!(stage_image(&bundle, 0.6) == Some(frames[2]));
        // wraps
        assert!(stage_image(&bundle, 0.8) == Some(frames[0]));
        // negative shader time clamps to frame 0
        assert!(stage_image(&bundle, -2.0) == Some(frames[0]));
    }

    #[test]
    fn a_one_shot_animmap_sticks_on_its_last_frame() {
        let frames: Vec<ImageHandle> = (0..3).map(|i| ImageHandle::new(i, 0)).collect();
        let bundle = TextureBundle {
            image: Some(frames[0]),
            num_image_animations: 3,
            image_animation_speed: 4.0,
            one_shot_anim_map: true,
            image_animations: frames.clone(),
            ..empty_bundle()
        };
        assert!(stage_image(&bundle, 5.0) == Some(frames[2]));
    }

    #[test]
    fn a_videomap_bundle_binds_nothing() {
        let bundle = TextureBundle {
            image: Some(ImageHandle::new(1, 0)),
            is_video_map: true,
            ..empty_bundle()
        };
        assert!(stage_image(&bundle, 0.0).is_none());
    }

    /// `NUM_TEXTURE_BUNDLES` bundles per stage — backend #1 draws bundle 0
    /// only, so the constant is asserted rather than silently assumed.
    #[test]
    fn a_stage_carries_the_oracles_bundle_count() {
        assert_eq!(
            stage(colorGen_t::CGEN_IDENTITY, alphaGen_t::AGEN_IDENTITY)
                .bundle
                .len(),
            NUM_TEXTURE_BUNDLES
        );
    }
}
