//! Raven `tr_init.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_init.cpp`

#![allow(non_snake_case)]

use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::cvar_fns::{Cvar_Get, Cvar_Set, Cvar_VariableString};
use mp_qshared::common::mp::cgame::texture_compression_t::textureCompression_t;
use mp_qshared::shared::cvar::{CvarHandle, CVAR_ROM};
use mp_qshared::shared::q_color::S_COLOR_YELLOW;

use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_assets_sim::RenderAssetsSim;
use crate::render_state::renderer_cvars::RendererCvars;

/// `VidModeTable` — the built-in video-mode list `R_GetModeInfo`/
/// `R_ModeList_f` index by small integer (DEC-37 A13.3 — NAMED BY THIS WAVE:
/// no R2 row assigns `r_vidModes`/`s_numVidModes` a home, and no earlier
/// packet carries them). Render-side, genuinely-const per subsystem-init
/// (§B3/B6): filled once when the renderer starts, matching Raven's
/// file-scope `static const vidmode_t r_vidModes[]`.
///
/// `s_numVidModes` is [`VidModeTable::modes`]`.len()`.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:465-480`
pub struct VidModeTable {
    pub modes: Vec<VidMode>,
}

impl Default for VidModeTable {
    /// Raven `const vidmode_t r_vidModes[]`.
    ///
    /// Source: `oracle/codemp/renderer/tr_init.cpp:465-479`
    fn default() -> VidModeTable {
        VidModeTable {
            modes: [
                ("Mode  0: 320x240", 320, 240),
                ("Mode  1: 400x300", 400, 300),
                ("Mode  2: 512x384", 512, 384),
                ("Mode  3: 640x480", 640, 480),
                ("Mode  4: 800x600", 800, 600),
                ("Mode  5: 960x720", 960, 720),
                ("Mode  6: 1024x768", 1024, 768),
                ("Mode  7: 1152x864", 1152, 864),
                ("Mode  8: 1280x1024", 1280, 1024),
                ("Mode  9: 1600x1200", 1600, 1200),
                ("Mode 10: 2048x1536", 2048, 1536),
                ("Mode 11: 856x480 (wide)", 856, 480),
                ("Mode 12: 2400x600(surround)", 2400, 600),
            ]
            .into_iter()
            .map(|(description, width, height)| VidMode {
                description: description.to_string(),
                width,
                height,
            })
            .collect(),
        }
    }
}

/// Raven `vidmode_t` — one entry of [`VidModeTable`].
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:459-463`
pub struct VidMode {
    pub description: String,
    pub width: i32,
    pub height: i32,
}

/// Raven `AssertCvarRange`.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:300-321`
pub fn AssertCvarRange(
    view: &mut EngineHostView,
    cv: CvarHandle,
    min_val: f32,
    max_val: f32,
    should_be_integral: bool,
) {
    let warn = S_COLOR_YELLOW.to_str().expect("S_COLOR_YELLOW is ASCII");
    let (name, integer) = {
        let c = view.common.cvar(cv);
        (c.name.clone(), c.integer)
    };

    if should_be_integral {
        let value = view.common.cvar(cv).value;
        if value as i32 != integer {
            com_printf(
                view.common,
                &format!(
                    "{}WARNING: cvar '{}' must be integral ({:.6})\n",
                    warn, name, value
                ),
            );
            Cvar_Set(view, &name, &format!("{}", integer));
        }
    }

    let value = view.common.cvar(cv).value;
    if value < min_val {
        com_printf(
            view.common,
            &format!(
                "{}WARNING: cvar '{}' out of range ({:.6} < {:.6})\n",
                warn, name, value, min_val
            ),
        );
        Cvar_Set(view, &name, &format!("{:.6}", min_val));
    } else if value > max_val {
        com_printf(
            view.common,
            &format!(
                "{}WARNING: cvar '{}' out of range ({:.6} > {:.6})\n",
                warn, name, value, max_val
            ),
        );
        Cvar_Set(view, &name, &format!("{:.6}", max_val));
    }
}

// DEFERRED: R4 — `GL_CheckErrors` is fixed-function-GL-surface only
// (`qglGetError` plus an enum→string switch feeding straight into
// `Com_Error`); no CPU logic survives independent of that GL call. R2 leaves
// `qgl*`/`qwgl*` entry points unhomed until the R4 wgpu rewrite (DEC-01/
// DEC-37 A13.2) — a frontend fn must not grow a GL dependency, so no stub
// body is written here. `r_ignoreGLErrors` (`RendererCvars`) is the only
// non-GL state this fn touched.
// Source: oracle/codemp/renderer/tr_init.cpp:414-450

/// Raven `R_GetModeInfo`.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:483-505`
pub fn R_GetModeInfo(
    common: &Common,
    cvars: &RendererCvars,
    vidmodes: &VidModeTable,
    mode: i32,
) -> Option<(i32, i32)> {
    if mode < -1 {
        return None;
    }
    if mode >= vidmodes.modes.len() as i32 {
        return None;
    }

    if mode == -1 {
        let width = common.cvar(cvars.r_customwidth).integer;
        let height = common.cvar(cvars.r_customheight).integer;
        return Some((width, height));
    }

    let vm = &vidmodes.modes[mode as usize];
    Some((vm.width, vm.height))
}

/// Raven `R_ModeList_f`.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:510-520`
pub fn R_ModeList_f(common: &mut Common, vidmodes: &VidModeTable) {
    com_printf(common, "\n");
    for vm in &vidmodes.modes {
        com_printf(common, &format!("{}\n", vm.description));
    }
    com_printf(common, "\n");
}

/// Raven `R_ScreenshotFilename`.
///
/// Out-params `fileName`/`MAX_OSPATH` cap collapse to an owned return
/// (§C7); `Com_sprintf`/`va` → `format!` per the translation dictionary.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:603-621`
pub fn R_ScreenshotFilename(last_number: i32, ext: &str) -> String {
    if !(0..=9999).contains(&last_number) {
        return format!("screenshots/shot9999{}", ext);
    }

    let mut n = last_number;
    let a = n / 1000;
    n -= a * 1000;
    let b = n / 100;
    n -= b * 100;
    let c = n / 10;
    n -= c * 10;
    let d = n;

    format!("screenshots/shot{}{}{}{}{}", a, b, c, d, ext)
}

/// Raven `GfxInfo_f`.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:875-972`
pub fn GfxInfo_f(view: &mut EngineHostView, cvars: &RendererCvars, assets: &RenderAssets) {
    let onoff = |b: bool| if b { "enabled" } else { "disabled" };

    let sys_cpustring = Cvar_Get(view, "sys_cpustring", "", CVAR_ROM);

    // PORT-NOTE: `RenderAssets::glconfig` (`GlConfig`) mirrors tier-1
    // `glconfig_t`'s field set as owned `String`/scalar members — its
    // `c_char` pointers are forbidden interior (R2 `## State ownership`
    // `glConfig` row: "not the tier-1 `glconfig_t`… `String`s here"). This
    // wave, the one the `glConfig` R2 row names as landing site, reads them
    // under that shape.
    let glconfig = &assets.glconfig;

    com_printf(
        view.common,
        &format!(
            "\nGL_VENDOR: {}\nGL_RENDERER: {}\nGL_VERSION: {}\nGL_EXTENSIONS: {}\n",
            glconfig.vendor_string,
            glconfig.renderer_string,
            glconfig.version_string,
            glconfig.extensions_string
        ),
    );
    com_printf(
        view.common,
        &format!(
            "GL_MAX_TEXTURE_SIZE: {}\nGL_MAX_ACTIVE_TEXTURES_ARB: {}\n",
            glconfig.max_texture_size, glconfig.max_active_textures
        ),
    );
    com_printf(
        view.common,
        &format!(
            "\nPIXELFORMAT: color({}-bits) Z({}-bit) stencil({}-bits)\n",
            glconfig.color_bits, glconfig.depth_bits, glconfig.stencil_bits
        ),
    );

    let r_mode = view.common.cvar(cvars.r_mode).integer;
    let r_fullscreen = view.common.cvar(cvars.r_fullscreen).integer;
    let fsstring = if r_fullscreen == 1 {
        "fullscreen"
    } else {
        "windowed"
    };
    com_printf(
        view.common,
        &format!(
            "MODE: {}, {} x {} {} hz:",
            r_mode, glconfig.vid_width, glconfig.vid_height, fsstring
        ),
    );
    let display_frequency = glconfig.display_frequency;
    if display_frequency != 0 {
        com_printf(view.common, &format!("{}\n", display_frequency));
    } else {
        com_printf(view.common, "N/A\n");
    }

    // DEFERRED: `tr.overbrightBits` is `trGlobals_t` frontend scratch ->
    // `RenderWorld::frame: FrameState` (`## State ownership` "tr frontend
    // scratch/counters" row); the field is not yet landed on `FrameState`
    // and this wave does not own that struct — the GAMMA line is skipped
    // whole rather than printing a placeholder value (no speculative
    // behavior, porting-rules §A2).
    // Source: oracle/codemp/renderer/tr_init.cpp:912-919

    let cpu_speed = Cvar_VariableString(view.common, "sys_cpuspeed").to_string();
    let cpu_string = view.common.cvar(sys_cpustring).string.clone();
    com_printf(
        view.common,
        &format!("CPU: {} @ {} MHz\n", cpu_string, cpu_speed),
    );

    // rendering primitives
    {
        let primitives = view.common.cvar(cvars.r_primitives).integer;
        com_printf(view.common, "rendering primitives: ");
        if primitives == 0 {
            // DEFERRED: R4 — `qglLockArraysEXT` has no R3 home (DEC-37
            // A13.2); the primitives==0 auto-select ("2 if compiled vertex
            // arrays are present, else 1") needs that GL entry point, so the
            // fallback resolution is skipped here.
            // Source: oracle/codemp/renderer/tr_init.cpp:929-935
        }
        match primitives {
            -1 => com_printf(view.common, "none\n"),
            2 => com_printf(view.common, "single glDrawElements\n"),
            1 => com_printf(view.common, "multiple glArrayElement\n"),
            3 => com_printf(
                view.common,
                "multiple glColor4ubv + glTexCoord2fv + glVertex3fv\n",
            ),
            _ => {}
        }
    }

    let texture_mode = view.common.cvar(cvars.r_textureMode).string.clone();
    com_printf(view.common, &format!("texturemode: {}\n", texture_mode));
    let picmip = view.common.cvar(cvars.r_picmip).integer;
    com_printf(view.common, &format!("picmip: {}\n", picmip));
    let texturebits = view.common.cvar(cvars.r_texturebits).integer;
    com_printf(view.common, &format!("texture bits: {}\n", texturebits));
    let texturebitslm = view.common.cvar(cvars.r_texturebitslm).integer;
    com_printf(
        view.common,
        &format!("lightmap texture bits: {}\n", texturebitslm),
    );

    // DEFERRED: R4 — `qglActiveTextureARB`/`qglLockArraysEXT` non-null
    // checks (the "multitexture"/"compiled vertex arrays" support lines)
    // have no R3 home (DEC-37 A13.2).
    // Source: oracle/codemp/renderer/tr_init.cpp:951-952

    com_printf(
        view.common,
        &format!(
            "texenv add: {}\n",
            onoff(glconfig.texture_env_add_available)
        ),
    );
    let compressed_textures = glconfig.texture_compression != textureCompression_t::TC_NONE;
    com_printf(
        view.common,
        &format!("compressed textures: {}\n", onoff(compressed_textures)),
    );

    let r_ext_compressed_lightmaps = view.common.cvar(cvars.r_ext_compressed_lightmaps).integer;
    let compressed_lightmaps = r_ext_compressed_lightmaps != 0
        && glconfig.texture_compression != textureCompression_t::TC_NONE;
    com_printf(
        view.common,
        &format!("compressed lightmaps: {}\n", onoff(compressed_lightmaps)),
    );

    let tc_name = match glconfig.texture_compression {
        textureCompression_t::TC_NONE => "None",
        textureCompression_t::TC_S3TC => "GL_S3_s3tc",
        textureCompression_t::TC_S3TC_DXT => "GL_EXT_texture_compression_s3tc",
    };
    com_printf(
        view.common,
        &format!("texture compression method: {}\n", tc_name),
    );

    let r_ext_aniso_integer = view
        .common
        .cvar(cvars.r_ext_texture_filter_anisotropic)
        .integer;
    let aniso_enabled = r_ext_aniso_integer != 0 && glconfig.max_texture_filter_anisotropy != 0.0;
    com_printf(
        view.common,
        &format!("anisotropic filtering: {}  ", onoff(aniso_enabled)),
    );
    let r_ext_aniso_value = view
        .common
        .cvar(cvars.r_ext_texture_filter_anisotropic)
        .value;
    com_printf(
        view.common,
        &format!(
            "({:.6} of {:.6})\n",
            r_ext_aniso_value, glconfig.max_texture_filter_anisotropy
        ),
    );

    let r_dynamic_glow = view.common.cvar(cvars.r_DynamicGlow).integer;
    // PORT-NOTE: the oracle indexes `enablestrings[r_DynamicGlow->integer]`
    // directly (no `!= 0` normalization, unlike every other enablestrings
    // use in this fn) — an out-of-range cvar value would be an oracle OOB
    // read (UB). Picking the `!= 0` boolean reading as the one defined
    // behavior (porting-rules §19).
    com_printf(
        view.common,
        &format!("Dynamic Glow: {}\n", onoff(r_dynamic_glow != 0)),
    );

    // DEFERRED: `g_bTextureRectangleHack` is not this TU's state — an
    // `extern` in the renderer sources with no definition anywhere in the
    // renderer TU set (STATE HOMES table: engine/client-side owner, confirm
    // at port time). The ATI-hack print line is skipped until that seam is
    // resolved.
    // Source: oracle/codemp/renderer/tr_init.cpp:960

    let r_finish = view.common.cvar(cvars.r_finish).integer;
    if r_finish != 0 {
        com_printf(view.common, "Forcing glFinish\n");
    }
    let r_display_refresh = view.common.cvar(cvars.r_displayRefresh).integer;
    if r_display_refresh != 0 {
        com_printf(
            view.common,
            &format!("Display refresh set to {}\n", r_display_refresh),
        );
    }

    // DEFERRED: `tr.world`/`lightGridSize` — `WorldAsset`'s fields land with
    // the `tr_bsp` R3 wave, not this `tr_init` wave (`render_state
    // /placeholders.rs` `WorldAsset` doc note); no field exists yet to read.
    // Source: oracle/codemp/renderer/tr_init.cpp:968-971
}

// DEFERRED: `R_AtiHackToggle_f` — `g_bTextureRectangleHack` is not this TU's
// state: an `extern` in the renderer sources with no definition anywhere in
// the renderer TU set (STATE HOMES table: engine/client-side owner, confirm
// at port time). No R3 renderer-owned field exists yet to toggle.
// Source: oracle/codemp/renderer/tr_init.cpp:974-977

/// Raven `RE_GetLightStyle`.
///
/// Already ported as [`RenderAssetsSim::get_light_style`] (R2 froze the
/// mutator/capacity/failure-value shape — `R2-D5`/`R2-D9`/`R2-D11`); this is
/// the Raven-named engine-boundary entry point the (deleted, DEC-37 ruling
/// 4) `refexport_t::GetLightStyle` slot would have pointed at. Out-param
/// `color4ub_t color` → an owned `[u8; 4]` return (§C7).
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:1427-1436`
pub fn RE_GetLightStyle(sim: &RenderAssetsSim, style: usize) -> [u8; 4] {
    sim.get_light_style(style)
}

/// Raven `RE_SetLightStyle`.
///
/// Already ported as [`RenderAssetsSim::set_light_style`] (`R2-D5`/`R2-D9`/
/// `R2-D11`); the Raven-named entry point the deleted `refexport_t
/// ::SetLightStyle` slot would have pointed at. Packed `int color` → an
/// owned `[u8; 4]` (§C7); the oracle's `if (*(int*)styleColors[style] !=
/// color)` guard is a pure write-elision optimization with no observable
/// difference from the unconditional write `set_light_style` already does.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:1438-1450`
pub fn RE_SetLightStyle(sim: &mut RenderAssetsSim, style: usize, color: [u8; 4]) {
    sim.set_light_style(style, color);
}

// DEFERRED: `GetRefAPI` — `refexport_t` is deleted at the Rust boundary
// (DEC-37 ruling 4: "direct calls / trait, no table"). The per-`RE_*`
// function bodies port individually in their owning waves (e.g.
// `RE_GetLightStyle`/`RE_SetLightStyle` above); no aggregate function-pointer
// export table survives to build, so `GetRefAPI` itself has no R3 body. Its
// fn-scope `static refexport_t re` dissolves with the type it held (the
// three-kind-rule classification is moot once the carrier is gone).
// Source: oracle/codemp/renderer/tr_init.cpp:1459-1531
