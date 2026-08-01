//! Raven `tr_init.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_init.cpp`

#![allow(non_snake_case)]

use core::f64::consts::PI;
use core::ffi::c_int;
use std::sync::Arc;

use mp_engine_qcommon::cmd_common::{Cmd_Argc, Cmd_Argv};
use mp_engine_qcommon::cmd_pc::Cmd_RemoveCommand;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::cvar_fns::{Cvar_Get, Cvar_Set, Cvar_VariableString};
use mp_engine_qcommon::files_common::FS_WriteFile;
use mp_engine_qcommon::qfiles::light_style_limits::MAX_LIGHT_STYLES;
use mp_qshared::common::mp::cgame::texture_compression_t::textureCompression_t;
use mp_qshared::shared::com_parse::QSharedScratch;
use mp_qshared::shared::cvar::{
    CvarHandle, CVAR_ARCHIVE, CVAR_CHEAT, CVAR_LATCH, CVAR_ROM, CVAR_TEMP,
};
use mp_qshared::shared::q_color::S_COLOR_YELLOW;
use native_math::rng::Rng;
use native_platform::Sys_LowPhysicalMemory;

use crate::gl_constants::GL_CLAMP;
use crate::render_state::frame_data::FrameData;
use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::placeholders::{
    BackEndCounters, OrientationR, RefEntity, TrRefdef, ViewParms, FUNCTABLE_SIZE,
};
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_assets_sim::RenderAssetsSim;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::tr_backend::{GL_Bind, GL_State, GL_TexEnv, RB_SetGL2D, RB_ShowImages};
use crate::tr_cmds::{r_init_command_buffers, r_shutdown_command_buffers, R_SyncRenderThread};
use crate::tr_font::{FontState, R_InitFonts, R_ShutdownFonts};
use crate::tr_image::{
    GL_TextureMode, R_DeleteTextures, R_FindImageFile, R_InitFogTable, R_InitImages, R_InitSkins,
    TrImageState,
};
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_model::render_models::RenderModels;
use crate::tr_noise::{NoiseState, R_NoiseInit};
use crate::tr_scene::{R_InitDecals, R_ToggleSmpFrame, SceneState};
use crate::tr_shader::{R_InitShaders, GLS_DSTBLEND_ZERO, GLS_SRCBLEND_ONE, GL_MODULATE};
use crate::tr_sky::SkyState;
use crate::tr_worldeffects::world_effects::WorldEffectsState;

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

/// Raven `R_TakeScreenshot`.
///
/// `Hunk_AllocateTempMemory`/`Hunk_FreeTempMemory` collapse to an owned local
/// `Vec<u8>` (porting-rules §C9 — the established `R_MipMap2` precedent,
/// `tr_image.rs:180-187`), never a raw-pointer alloc/free pair; the "swap rgb
/// to bgr" pointer walk becomes `Vec::swap` (same behavior, idiomatic shape,
/// porting-rules §C10).
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:537-571`
// `x`/`y` are read only by the R4-deferred `qglReadPixels` call below.
#[allow(unused_variables)]
pub fn R_TakeScreenshot(
    common: &mut Common,
    assets: &RenderAssets,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    file_name: &str,
) {
    let vid_width = assets.glconfig.vid_width;
    let vid_height = assets.glconfig.vid_height;

    // `Com_Memset(buffer, 0, 18)` — `vec![0u8; N]` is already zero-filled.
    let mut buffer = vec![0u8; (vid_width * vid_height * 3 + 18).max(0) as usize];
    buffer[2] = 2; // uncompressed type
    buffer[12] = (width & 255) as u8;
    buffer[13] = (width >> 8) as u8;
    buffer[14] = (height & 255) as u8;
    buffer[15] = (height >> 8) as u8;
    buffer[16] = 24; // pixel size

    // DEFERRED: R4 — `qglReadPixels(x, y, width, height, GL_RGB,
    // GL_UNSIGNED_BYTE, buffer+18)`: the fixed-function GL surface has no R3
    // home (DEC-01/DEC-37 A13.2 — `GpuResources::gl_state` is a named
    // placeholder until the wgpu rewrite). `buffer[18..]` stays zero-filled
    // until R4 fills it; the surrounding CPU logic (header, channel swap,
    // gamma, file write) is still ported per this wave's threading digest
    // ("port the CPU logic").
    // Source: oracle/codemp/renderer/tr_init.cpp:552

    // swap rgb to bgr
    let c = (18 + width * height * 3) as usize;
    let mut i = 18usize;
    while i < c {
        buffer.swap(i, i + 2);
        i += 3;
    }

    // DEFERRED: `tr.overbrightBits > 0` — `trGlobals_t` frontend scratch ->
    // `RenderWorld::frame: FrameState` (`## State ownership` "tr frontend
    // scratch/counters" row); the field is not yet landed on `FrameState`
    // (same gap `GfxInfo_f`'s GAMMA-line note above already flags), so the
    // gamma-correct gate can't be evaluated and the whole conditional is
    // skipped rather than guessed (porting-rules §A2).
    // Source: oracle/codemp/renderer/tr_init.cpp:562-565

    FS_WriteFile(common, file_name, buffer.as_ptr() as *const (), c as c_int);

    // `Hunk_FreeTempMemory(buffer)` — no-op: `buffer` (owned `Vec<u8>`) drops
    // here (porting-rules §C9).
}

// DEFERRED: `R_LevelShot` — two state-home gaps stack on this fn, neither
// licensed to invent:
// (1) `tr.world->baseName` — the `world_t` tier-2 transition audit's `world_t`
//     row promises "String x2 for the names" once `tr_bsp`/`tr_world` land,
//     but the currently-landed `WorldAsset` (`render_state/placeholders.rs`)
//     carries only `name`, no `base_name` field — no state home exists yet
//     (preamble: "leave a cited // DEFERRED: and raise it — do NOT create a
//     field"). It names the output file, so it blocks the very first
//     statement every other one depends on; no partial CPU-logic body is
//     written (unlike `R_TakeScreenshot` above).
// (2) `qglReadPixels` — same GL-surface gap as `R_TakeScreenshot` above
//     (DEC-01/DEC-37 A13.2).
// `LEVELSHOTSIZE` is `256` (oracle/codemp/renderer/tr_init.cpp:631) — no
// longer a gap; the constant lands with the body.
// Source: oracle/codemp/renderer/tr_init.cpp:632-691

/// Raven `R_Register`.
///
/// One `Cvar_Get` per `RendererCvars` field (DEC-37 A13.1); flag/default
/// values transcribed verbatim, including the oracle's own-looking quirks
/// (`"0.8f"`/`"1.13f"` default strings, the `"r_roofCeilFloorDist"` cvar name
/// under the `r_roofCullFloorDist` field — both already noted in
/// `renderer_cvars.rs`'s per-field doc comments, not re-derived here).
///
/// Platform `#ifdef` resolution follows the established MP-retail-build
/// precedent (non-`_XBOX`, non-`__linux__`, non-`__MACOS__`, non-`_DEBUG` —
/// `tr_shade_calc.rs:263-264`, `tr_shadows.rs:76`): the `_XBOX`-only cvars
/// (`r_hdreffect`/`r_sundir_*`/`r_hdrbloom`) and the `_DEBUG`-only
/// `r_noPrecacheGLA` are dropped, not registered.
///
/// The `#ifndef DEDICATED` command-registration block (`imagelist`/
/// `shaderlist`/`skinlist`/`screenshot`/`screenshot_tga`/`gfxinfo`/
/// `r_atihack`/`r_we`/`imagecacheinfo`) is the client leg and belongs here
/// (R3 client-leg ruling: the R3 renderer track is the CLIENT port; the
/// jampDed disposition is scoped to the dedicated-server link set). It is
/// **not** registered yet — see the `//TODO: Port R_Register console
/// commands` marker at the block's own position in the body below for the
/// blocker, which is the same `CmdFunction` signature mismatch the
/// unconditionally-registered `R_ModeList_f` marker beside it already
/// records.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:985-1205`
pub fn R_Register(view: &mut EngineHostView, cvars: &mut RendererCvars) {
    //
    // latched and archived variables
    //
    cvars.r_allowExtensions = Some(Cvar_Get(
        view,
        "r_allowExtensions",
        "1",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_ext_compressed_textures = Some(Cvar_Get(
        view,
        "r_ext_compress_textures",
        "1",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_ext_compressed_lightmaps = Some(Cvar_Get(
        view,
        "r_ext_compress_lightmaps",
        "0",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_ext_preferred_tc_method = Some(Cvar_Get(
        view,
        "r_ext_preferred_tc_method",
        "0",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_ext_gamma_control = Some(Cvar_Get(
        view,
        "r_ext_gamma_control",
        "1",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_ext_multitexture = Some(Cvar_Get(
        view,
        "r_ext_multitexture",
        "1",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_ext_compiled_vertex_array = Some(Cvar_Get(
        view,
        "r_ext_compiled_vertex_array",
        "1",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    // MP retail builds the non-`__linux__` branch (established precedent —
    // see doc comment above).
    cvars.r_ext_texture_env_add = Some(Cvar_Get(
        view,
        "r_ext_texture_env_add",
        "1",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_ext_texture_filter_anisotropic = Some(Cvar_Get(
        view,
        "r_ext_texture_filter_anisotropic",
        "16",
        CVAR_ARCHIVE,
    ));

    cvars.r_DynamicGlow = Some(Cvar_Get(view, "r_DynamicGlow", "0", CVAR_ARCHIVE));
    cvars.r_DynamicGlowPasses = Some(Cvar_Get(view, "r_DynamicGlowPasses", "5", CVAR_CHEAT));
    cvars.r_DynamicGlowDelta = Some(Cvar_Get(view, "r_DynamicGlowDelta", "0.8f", CVAR_CHEAT));
    cvars.r_DynamicGlowIntensity = Some(Cvar_Get(
        view,
        "r_DynamicGlowIntensity",
        "1.13f",
        CVAR_CHEAT,
    ));
    cvars.r_DynamicGlowSoft = Some(Cvar_Get(view, "r_DynamicGlowSoft", "1", CVAR_CHEAT));
    cvars.r_DynamicGlowWidth = Some(Cvar_Get(
        view,
        "r_DynamicGlowWidth",
        "320",
        CVAR_CHEAT | CVAR_LATCH,
    ));
    cvars.r_DynamicGlowHeight = Some(Cvar_Get(
        view,
        "r_DynamicGlowHeight",
        "240",
        CVAR_CHEAT | CVAR_LATCH,
    ));

    cvars.r_picmip = Some(Cvar_Get(view, "r_picmip", "1", CVAR_ARCHIVE | CVAR_LATCH));
    cvars.r_colorMipLevels = Some(Cvar_Get(view, "r_colorMipLevels", "0", CVAR_LATCH));
    AssertCvarRange(view, cvars.r_picmip.unwrap(), 0.0, 16.0, true);
    cvars.r_detailTextures = Some(Cvar_Get(
        view,
        "r_detailtextures",
        "1",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_texturebits = Some(Cvar_Get(
        view,
        "r_texturebits",
        "0",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_texturebitslm = Some(Cvar_Get(
        view,
        "r_texturebitslm",
        "0",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_colorbits = Some(Cvar_Get(
        view,
        "r_colorbits",
        "0",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_stereo = Some(Cvar_Get(view, "r_stereo", "0", CVAR_ARCHIVE | CVAR_LATCH));
    // MP retail builds the non-`__linux__` branch (established precedent —
    // see doc comment above).
    cvars.r_stencilbits = Some(Cvar_Get(
        view,
        "r_stencilbits",
        "8",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_depthbits = Some(Cvar_Get(
        view,
        "r_depthbits",
        "0",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_overBrightBits = Some(Cvar_Get(
        view,
        "r_overBrightBits",
        "0",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_ignorehwgamma = Some(Cvar_Get(
        view,
        "r_ignorehwgamma",
        "0",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_mode = Some(Cvar_Get(view, "r_mode", "4", CVAR_ARCHIVE | CVAR_LATCH));
    cvars.r_fullscreen = Some(Cvar_Get(
        view,
        "r_fullscreen",
        "1",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_customwidth = Some(Cvar_Get(
        view,
        "r_customwidth",
        "1600",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_customheight = Some(Cvar_Get(
        view,
        "r_customheight",
        "1024",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_simpleMipMaps = Some(Cvar_Get(
        view,
        "r_simpleMipMaps",
        "1",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_vertexLight = Some(Cvar_Get(
        view,
        "r_vertexLight",
        "0",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));
    cvars.r_uiFullScreen = Some(Cvar_Get(view, "r_uifullscreen", "0", 0));
    cvars.r_subdivisions = Some(Cvar_Get(
        view,
        "r_subdivisions",
        "4",
        CVAR_ARCHIVE | CVAR_LATCH,
    ));

    //
    // temporary latched variables that can only change over a restart
    //
    cvars.r_displayRefresh = Some(Cvar_Get(view, "r_displayRefresh", "0", CVAR_LATCH));
    AssertCvarRange(view, cvars.r_displayRefresh.unwrap(), 0.0, 200.0, true);
    cvars.r_fullbright = Some(Cvar_Get(view, "r_fullbright", "0", CVAR_CHEAT));
    cvars.r_intensity = Some(Cvar_Get(view, "r_intensity", "1", CVAR_LATCH));
    cvars.r_singleShader = Some(Cvar_Get(
        view,
        "r_singleShader",
        "0",
        CVAR_CHEAT | CVAR_LATCH,
    ));

    //
    // archived variables that can change at any time
    //
    cvars.r_lodCurveError = Some(Cvar_Get(view, "r_lodCurveError", "250", CVAR_ARCHIVE));
    cvars.r_lodbias = Some(Cvar_Get(view, "r_lodbias", "0", CVAR_ARCHIVE));
    cvars.r_autolodscalevalue = Some(Cvar_Get(view, "r_autolodscalevalue", "0", CVAR_ROM));

    cvars.r_flares = Some(Cvar_Get(view, "r_flares", "1", CVAR_ARCHIVE));
    // MP retail builds the non-`_XBOX` branch (established precedent — see
    // doc comment above).
    cvars.r_znear = Some(Cvar_Get(view, "r_znear", "4", CVAR_CHEAT));
    AssertCvarRange(view, cvars.r_znear.unwrap(), 0.001, 200.0, true);
    cvars.r_ignoreGLErrors = Some(Cvar_Get(view, "r_ignoreGLErrors", "1", CVAR_ARCHIVE));
    cvars.r_fastsky = Some(Cvar_Get(view, "r_fastsky", "0", CVAR_ARCHIVE));
    cvars.r_inGameVideo = Some(Cvar_Get(view, "r_inGameVideo", "1", CVAR_ARCHIVE));
    cvars.r_drawSun = Some(Cvar_Get(view, "r_drawSun", "0", CVAR_ARCHIVE));
    cvars.r_dynamiclight = Some(Cvar_Get(view, "r_dynamiclight", "1", CVAR_ARCHIVE));
    // rjr - removed for hacking r_dlightBacks = Cvar_Get( "r_dlightBacks", "1", CVAR_CHEAT );
    cvars.r_finish = Some(Cvar_Get(view, "r_finish", "0", CVAR_ARCHIVE));
    cvars.r_textureMode = Some(Cvar_Get(
        view,
        "r_textureMode",
        "GL_LINEAR_MIPMAP_NEAREST",
        CVAR_ARCHIVE,
    ));
    cvars.r_swapInterval = Some(Cvar_Get(view, "r_swapInterval", "0", CVAR_ARCHIVE));
    cvars.r_markcount = Some(Cvar_Get(view, "r_markcount", "100", CVAR_ARCHIVE));
    // MP retail builds the non-`__MACOS__` branch (established precedent —
    // see doc comment above).
    cvars.r_gamma = Some(Cvar_Get(view, "r_gamma", "1", CVAR_ARCHIVE));
    cvars.r_facePlaneCull = Some(Cvar_Get(view, "r_facePlaneCull", "1", CVAR_ARCHIVE));

    // attempted smart method of culling out upwards facing surfaces on
    // roofs for automap shots -rww
    cvars.r_cullRoofFaces = Some(Cvar_Get(view, "r_cullRoofFaces", "0", CVAR_CHEAT));
    cvars.r_roofCullCeilDist = Some(Cvar_Get(view, "r_roofCullCeilDist", "256", CVAR_CHEAT));
    cvars.r_roofCullFloorDist = Some(Cvar_Get(view, "r_roofCeilFloorDist", "128", CVAR_CHEAT));

    cvars.r_primitives = Some(Cvar_Get(view, "r_primitives", "0", CVAR_ARCHIVE));

    cvars.r_ambientScale = Some(Cvar_Get(view, "r_ambientScale", "0.6", CVAR_CHEAT));
    cvars.r_directedScale = Some(Cvar_Get(view, "r_directedScale", "1", CVAR_CHEAT));

    // automap renderside toggle for debugging -rww
    cvars.r_autoMap = Some(Cvar_Get(view, "r_autoMap", "0", CVAR_ARCHIVE));
    // alpha of automap bg -rww
    cvars.r_autoMapBackAlpha = Some(Cvar_Get(view, "r_autoMapBackAlpha", "0", 0));
    cvars.r_autoMapDisable = Some(Cvar_Get(view, "r_autoMapDisable", "1", 0));

    //
    // temporary variables that can change at any time
    //
    cvars.r_showImages = Some(Cvar_Get(view, "r_showImages", "0", CVAR_CHEAT));

    cvars.r_debugLight = Some(Cvar_Get(view, "r_debuglight", "0", CVAR_TEMP));
    cvars.r_debugSort = Some(Cvar_Get(view, "r_debugSort", "0", CVAR_CHEAT));

    cvars.r_dlightStyle = Some(Cvar_Get(view, "r_dlightStyle", "1", CVAR_TEMP));
    cvars.r_surfaceSprites = Some(Cvar_Get(view, "r_surfaceSprites", "1", CVAR_TEMP));
    cvars.r_surfaceWeather = Some(Cvar_Get(view, "r_surfaceWeather", "0", CVAR_TEMP));

    cvars.r_windSpeed = Some(Cvar_Get(view, "r_windSpeed", "0", 0));
    cvars.r_windAngle = Some(Cvar_Get(view, "r_windAngle", "0", 0));
    cvars.r_windGust = Some(Cvar_Get(view, "r_windGust", "0", 0));
    cvars.r_windDampFactor = Some(Cvar_Get(view, "r_windDampFactor", "0.1", 0));
    cvars.r_windPointForce = Some(Cvar_Get(view, "r_windPointForce", "0", 0));
    cvars.r_windPointX = Some(Cvar_Get(view, "r_windPointX", "0", 0));
    cvars.r_windPointY = Some(Cvar_Get(view, "r_windPointY", "0", 0));

    cvars.r_nocurves = Some(Cvar_Get(view, "r_nocurves", "0", CVAR_CHEAT));
    cvars.r_drawworld = Some(Cvar_Get(view, "r_drawworld", "1", CVAR_CHEAT));
    cvars.r_drawfog = Some(Cvar_Get(view, "r_drawfog", "2", CVAR_CHEAT));
    cvars.r_lightmap = Some(Cvar_Get(view, "r_lightmap", "0", CVAR_CHEAT));
    cvars.r_portalOnly = Some(Cvar_Get(view, "r_portalOnly", "0", CVAR_CHEAT));

    cvars.r_skipBackEnd = Some(Cvar_Get(view, "r_skipBackEnd", "0", CVAR_CHEAT));

    cvars.r_measureOverdraw = Some(Cvar_Get(view, "r_measureOverdraw", "0", CVAR_CHEAT));
    cvars.r_lodscale = Some(Cvar_Get(view, "r_lodscale", "5", 0));
    cvars.r_norefresh = Some(Cvar_Get(view, "r_norefresh", "0", CVAR_CHEAT));
    cvars.r_drawentities = Some(Cvar_Get(view, "r_drawentities", "1", CVAR_CHEAT));
    cvars.r_ignore = Some(Cvar_Get(view, "r_ignore", "1", CVAR_CHEAT));
    cvars.r_nocull = Some(Cvar_Get(view, "r_nocull", "0", CVAR_CHEAT));
    cvars.r_novis = Some(Cvar_Get(view, "r_novis", "0", CVAR_CHEAT));
    cvars.r_showcluster = Some(Cvar_Get(view, "r_showcluster", "0", CVAR_CHEAT));
    cvars.r_speeds = Some(Cvar_Get(view, "r_speeds", "0", CVAR_CHEAT));
    cvars.r_verbose = Some(Cvar_Get(view, "r_verbose", "0", CVAR_CHEAT));
    cvars.r_logFile = Some(Cvar_Get(view, "r_logFile", "0", CVAR_CHEAT));
    cvars.r_debugSurface = Some(Cvar_Get(view, "r_debugSurface", "0", CVAR_CHEAT));
    cvars.r_nobind = Some(Cvar_Get(view, "r_nobind", "0", CVAR_CHEAT));
    cvars.r_showtris = Some(Cvar_Get(view, "r_showtris", "0", CVAR_CHEAT));
    cvars.r_showsky = Some(Cvar_Get(view, "r_showsky", "0", CVAR_CHEAT));
    cvars.r_shownormals = Some(Cvar_Get(view, "r_shownormals", "0", CVAR_CHEAT));
    cvars.r_clear = Some(Cvar_Get(view, "r_clear", "0", CVAR_CHEAT));
    cvars.r_offsetFactor = Some(Cvar_Get(view, "r_offsetfactor", "-1", CVAR_CHEAT));
    cvars.r_offsetUnits = Some(Cvar_Get(view, "r_offsetunits", "-2", CVAR_CHEAT));
    cvars.r_lockpvs = Some(Cvar_Get(view, "r_lockpvs", "0", CVAR_CHEAT));
    cvars.r_noportals = Some(Cvar_Get(view, "r_noportals", "0", CVAR_CHEAT));
    cvars.r_shadows = Some(Cvar_Get(view, "cg_shadows", "1", 0));
    cvars.r_shadowRange = Some(Cvar_Get(view, "r_shadowRange", "1000", 0));

    // _XBOX-only cvars (r_hdreffect/r_sundir_x/r_sundir_y/r_sundir_z/
    // r_hdrbloom) dropped — MP retail builds the non-`_XBOX` branch.
    // Source: oracle/codemp/renderer/tr_init.cpp:1142-1148

    // PORT-NOTE: `va("%d", MAX_POLYS)`/`va("%d", MAX_POLYVERTS)` -> literal
    // decimal strings; values corroborated by `RenderAssets::max_polys`/
    // `max_polyverts`'s own doc comments (`render_state/render_assets.rs:
    // 121-133`: "default MAX_POLYS = 600" / "default MAX_POLYVERTS = 3000"),
    // not guessed.
    cvars.r_maxpolys = Some(Cvar_Get(view, "r_maxpolys", "600", 0));
    cvars.r_maxpolyverts = Some(Cvar_Get(view, "r_maxpolyverts", "3000", 0));
    /*
    Ghoul2 Insert Start
    */
    // `r_noPrecacheGLA` (`_DEBUG`-only) dropped — MP retail builds the
    // non-`_DEBUG` branch.
    // Source: oracle/codemp/renderer/tr_init.cpp:1155-1157

    cvars.r_noServerGhoul2 = Some(Cvar_Get(view, "r_noserverghoul2", "0", CVAR_CHEAT));

    cvars.r_Ghoul2AnimSmooth = Some(Cvar_Get(view, "r_ghoul2animsmooth", "0.3", 0));
    cvars.r_Ghoul2UnSqashAfterSmooth = Some(Cvar_Get(view, "r_ghoul2unsqashaftersmooth", "1", 0));

    cvars.broadsword = Some(Cvar_Get(view, "broadsword", "0", 0));
    cvars.broadsword_kickbones = Some(Cvar_Get(view, "broadsword_kickbones", "1", 0));
    cvars.broadsword_kickorigin = Some(Cvar_Get(view, "broadsword_kickorigin", "1", 0));
    cvars.broadsword_dontstopanim = Some(Cvar_Get(view, "broadsword_dontstopanim", "0", 0));
    cvars.broadsword_waitforshot = Some(Cvar_Get(view, "broadsword_waitforshot", "0", 0));
    cvars.broadsword_playflop = Some(Cvar_Get(view, "broadsword_playflop", "1", 0));
    cvars.broadsword_smallbbox = Some(Cvar_Get(view, "broadsword_smallbbox", "0", 0));
    cvars.broadsword_extra1 = Some(Cvar_Get(view, "broadsword_extra1", "0", 0));
    cvars.broadsword_extra2 = Some(Cvar_Get(view, "broadsword_extra2", "0", 0));
    cvars.broadsword_effcorr = Some(Cvar_Get(view, "broadsword_effcorr", "1", 0));
    cvars.broadsword_ragtobase = Some(Cvar_Get(view, "broadsword_ragtobase", "2", 0));
    cvars.broadsword_dircap = Some(Cvar_Get(view, "broadsword_dircap", "64", 0));
    /*
    Ghoul2 Insert End
    */

    cvars.r_modelpoolmegs = Some(Cvar_Get(view, "r_modelpoolmegs", "20", CVAR_ARCHIVE));
    if Sys_LowPhysicalMemory() != 0 {
        Cvar_Set(view, "r_modelpoolmegs", "0");
    }

    // make sure all the commands added here are also removed in R_Shutdown

    //TODO: Port R_Register console commands
    // Source: oracle/codemp/renderer/tr_init.cpp:1188-1197
    // The nine `#ifndef DEDICATED` registrations (imagelist/shaderlist/
    // skinlist/screenshot/screenshot_tga/gfxinfo/r_atihack/r_we/
    // imagecacheinfo) are the client leg and belong here (R3 client-leg
    // ruling), superseding this file's earlier "DEDICATED is this build's
    // live configuration" rationale. `RE_Shutdown` below already removes all
    // nine, matching Raven's own comment above ("make sure all the commands
    // added here are also removed in R_Shutdown"), so leaving them
    // unregistered is the inconsistency this marker names.
    //
    // Blocker: `CmdFunction = fn(&mut EngineHostView)`
    // (`crates/mp/engine/qcommon/src/cmd/cmd_function_t.rs:12`) carries no
    // renderer state, and every one of the eight ported handlers needs some
    // (`R_ImageList_f`/`R_ShaderList_f`/`R_SkinList_f`/`R_ScreenShot_f`/
    // `R_ScreenShotTGA_f`: `&RenderAssets`; `GfxInfo_f`: `&RendererCvars` +
    // `&RenderAssets`; `RE_RegisterImages_Info_f`: `&RenderAssets` +
    // `&RenderModels`; `R_WorldEffect_f`: `&mut WorldEffectsState` + five
    // more). This is the same missing renderer-state-carrying adapter the
    // `R_ModeList_f` marker below already records.
    //
    // (i) The ninth handler, `R_AtiHackToggle_f`, is R4-deferred: its only
    // state is the GL-upload flag `g_bTextureRectangleHack`
    // (oracle/codemp/renderer/tr_init.cpp:974-977), read by the R4 upload
    // path and carried by no R2/R3 struct, so there is nothing for a ported
    // handler to toggle yet.
    //
    // (ii) The registration wiring itself is blocked on the client boot seam
    // owning a renderer instance reachable from `EngineHostView` (#46): every
    // handler above takes the DEC-42.3 client bundle (`&mut EngineHostView`
    // plus the renderer state), and nothing hands a `Cmd_AddCommand` callback
    // that state today.

    //TODO: Port R_Modellist_f
    // Source: oracle/codemp/renderer/tr_init.cpp:1199
    // Registered unconditionally (outside `#ifndef DEDICATED`) as
    // "modellist" — not yet ported anywhere in this crate.

    //TODO: Port R_ModeList_f Cmd_AddCommand wiring
    // Source: oracle/codemp/renderer/tr_init.cpp:1201
    // `R_ModeList_f` (this file — `common: &mut Common, vidmodes:
    // &VidModeTable`) is already ported but does not fit `CmdFunction =
    // fn(&mut EngineHostView)` (`crates/mp/engine/qcommon/src/cmd/
    // cmd_function_t.rs:12`) — no renderer-state-carrying adapter is
    // licensed by this packet's resolved call surface.

    //TODO: Port RE_RegisterModels_Info_f
    // Source: oracle/codemp/renderer/tr_init.cpp:1203
    // Registered unconditionally (outside `#ifndef DEDICATED`) as
    // "modelcacheinfo" — not yet ported anywhere in this crate.
}

/// Raven `R_TakeScreenshotJPEG`.
///
/// `Hunk_AllocateTempMemory`/`Hunk_FreeTempMemory` collapse to an owned
/// local `Vec<u8>`, matching the `R_TakeScreenshot` precedent above
/// (porting-rules §C9).
///
/// The gamma-correct gate (`tr.overbrightBits > 0 && glConfig
/// .deviceSupportsGamma`) needs `tr.overbrightBits` — `trGlobals_t`
/// frontend scratch -> `RenderWorld::frame: FrameState` (`## State
/// ownership` "tr frontend scratch/counters" row); not yet landed on
/// `FrameState` (same gap `GfxInfo_f`/`R_TakeScreenshot` above already
/// flag) — the whole conditional (and its `R_GammaCorrect` call) is
/// skipped rather than guessed (porting-rules §A2).
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:578-596`
// `x`/`y`/`width`/`height` are read only by the R4-deferred `qglReadPixels`
// call below.
#[allow(unused_variables)]
pub fn R_TakeScreenshotJPEG(
    common: &mut Common,
    assets: &RenderAssets,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    file_name: &str,
) {
    let vid_width = assets.glconfig.vid_width;
    let vid_height = assets.glconfig.vid_height;

    let buffer = vec![0u8; (vid_width * vid_height * 4).max(0) as usize];

    // DEFERRED: R4 — `qglReadPixels( x, y, width, height, GL_RGBA,
    // GL_UNSIGNED_BYTE, buffer )`: the fixed-function GL surface has no R3
    // home (DEC-01/DEC-37 A13.2 — `GpuResources::gl_state` is a named
    // placeholder until the wgpu rewrite). `buffer` stays zero-filled until
    // R4 fills it; the surrounding CPU logic is still ported per this
    // wave's threading digest ("port the CPU logic").
    // Source: oracle/codemp/renderer/tr_init.cpp:584

    // gamma correct — DEFERRED: `tr.overbrightBits`, see doc comment above.
    // Source: oracle/codemp/renderer/tr_init.cpp:586-589

    // `FS_WriteFile( fileName, buffer, 1 )` — "create path": the oracle
    // writes a single byte of `buffer` ahead of `SaveJPG`'s own write,
    // which creates the destination file/path `SaveJPG` then reopens.
    // Transcribed literally (size `1`, not the full buffer).
    FS_WriteFile(common, file_name, buffer.as_ptr() as *const (), 1);

    // DEFERRED: `SaveJPG` — entirely a vendored-libjpeg compression
    // pipeline with no Rust-crate jpeg-encode seam wired in this workspace
    // (see the `SaveJPG` DEFERRED-WHOLE block, `tr_image.rs:1338-1352`,
    // which this caller inherits verbatim per its own note: "extending
    // wave 0's jpegDest-family precedent... to the caller").
    // Source: oracle/codemp/renderer/tr_init.cpp:592

    // `Hunk_FreeTempMemory(buffer)` — no-op: `buffer` (owned `Vec<u8>`)
    // drops here (porting-rules §C9).
}

/// Raven `R_ScreenShotTGA_f`.
///
/// The "levelshot" branch calls `R_LevelShot`, which — despite this wave's
/// RESOLVED CALL SURFACE listing it as already landed in wave 1
/// (`oracle/codemp/renderer/tr_init.cpp:632-691`) — is actually DEFERRED
/// WHOLE above (no callable Rust fn exists; blocked on `tr.world
/// ->baseName`'s missing state home plus the R4 `qglReadPixels` gap): a
/// wave-planning discrepancy, raised as an escalation rather than silently
/// reconciled. The free-filename-scan `else` branch needs the fn-scope
/// static `lastNumber` (`static int lastNumber = -1`), classified genuine
/// cross-frame state (kind 3, three-kind rule) with NO R2 carrier assigned
/// (preamble: "a kind-3 static is an escalation… never an invented
/// field"). Both blocking points are transcribed as `todo!()` at the exact
/// site that needs them — the `GL_TextureMode`
/// `modes[6]`/`Taiwanese_CollapseBig5Code` precedent: transcribe
/// everything computable, block only at the dependency itself — rather
/// than deferring the whole function; the explicit-filename path
/// (`Cmd_Argc() == 2 && !silent`) has neither gap and runs for real.
///
/// `Com_sprintf`/`va` -> `format!` per the translation dictionary (the
/// `R_ScreenshotFilename` precedent above).
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:705-759`
pub fn R_ScreenShotTGA_f(view: &mut EngineHostView, assets: &RenderAssets) {
    if Cmd_Argv(view.common, 1) == "levelshot" {
        todo!(
            "Port R_LevelShot call — R_LevelShot is DEFERRED WHOLE above (tr_init.rs); oracle/codemp/renderer/tr_init.cpp:711-714"
        )
    }

    let silent = Cmd_Argv(view.common, 1) == "silent";

    let checkname = if Cmd_Argc(view.common) == 2 && !silent {
        // explicit filename
        format!("screenshots/{}.tga", Cmd_Argv(view.common, 1))
    } else {
        // scan for a free filename
        todo!(
            "Port R_ScreenShotTGA_f's free-filename scan — needs fn-scope static `lastNumber` (kind-3, no R2 carrier assigned); oracle/codemp/renderer/tr_init.cpp:722-749"
        )
    };

    R_TakeScreenshot(
        view.common,
        assets,
        0,
        0,
        assets.glconfig.vid_width,
        assets.glconfig.vid_height,
        &checkname,
    );

    if !silent {
        com_printf(view.common, &format!("Wrote {}\n", checkname));
    }
}

/// Raven `R_ScreenShot_f`.
///
/// Mirrors the `R_ScreenShotTGA_f` precedent above (same fn-scope-static and
/// `R_LevelShot`-deferred-whole gaps, same `todo!()`-at-the-site treatment
/// rather than deferring the whole function): the "levelshot" branch calls
/// `R_LevelShot`, DEFERRED WHOLE above (no callable Rust fn exists — blocked
/// on `tr.world->baseName`'s missing state home plus the R4 `qglReadPixels`
/// gap). The free-filename-scan `else` branch needs the fn-scope static
/// `lastNumber` (`static int lastNumber = -1`), classified genuine
/// cross-frame state (kind 3, three-kind rule) with NO R2 carrier assigned
/// (preamble: "a kind-3 static is an escalation… never an invented field").
/// Both blocking points are transcribed as `todo!()` at the exact site that
/// needs them; the explicit-filename path (`Cmd_Argc() == 2 && !silent`) has
/// neither gap and runs for real.
///
/// `Com_sprintf`/`va` -> `format!` per the translation dictionary (the
/// `R_ScreenshotFilename`/`R_ScreenShotTGA_f` precedent).
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:762-815`
pub fn R_ScreenShot_f(view: &mut EngineHostView, assets: &RenderAssets) {
    if Cmd_Argv(view.common, 1) == "levelshot" {
        todo!(
            "Port R_LevelShot call — R_LevelShot is DEFERRED WHOLE above (tr_init.rs); oracle/codemp/renderer/tr_init.cpp:768-770"
        )
    }

    let silent = Cmd_Argv(view.common, 1) == "silent";

    let checkname = if Cmd_Argc(view.common) == 2 && !silent {
        // explicit filename
        format!("screenshots/{}.jpg", Cmd_Argv(view.common, 1))
    } else {
        // scan for a free filename
        //
        // if we have saved a previous screenshot, don't scan again, because
        // recording demo avis can involve thousands of shots
        todo!(
            "Port R_ScreenShot_f's free-filename scan — needs fn-scope static `lastNumber` (kind-3, no R2 carrier assigned); oracle/codemp/renderer/tr_init.cpp:787-805"
        )
    };

    R_TakeScreenshotJPEG(
        view.common,
        assets,
        0,
        0,
        assets.glconfig.vid_width,
        assets.glconfig.vid_height,
        &checkname,
    );

    if !silent {
        com_printf(view.common, &format!("Wrote {}\n", checkname));
    }
}

/// Raven `GL_SetDefaultState`.
///
/// Every `qgl*` call in this fn (`qglClearDepth`/`qglCullFace`/
/// `qglColor4f`/`qglDisable`/`qglEnable`/`qglEnableClientState`/
/// `qglShadeModel`/`qglDepthFunc`/`qglPolygonMode`/`qglDepthMask`) is the
/// fixed-function GL surface DEC-01/DEC-37 leave unhomed until the R4 wgpu
/// rewrite (A13.2) — each is left as a cited `// DEFERRED: R4` rather than
/// a stub body. The `qglActiveTextureARB` non-null multitexture-support
/// gate is the same kind of gap (STATE HOMES: no R3 home), so its whole
/// guarded block (the `GL_SelectTexture`/`GL_TextureMode`/`GL_TexEnv`/
/// `qglDisable` sequence) is skipped rather than guessed at (porting-rules
/// §A2) — the unconditional `GL_TextureMode`/`GL_TexEnv` calls below it
/// still run. `glState.glStateBits` writes into `GpuResources::gl_state`
/// (`GlStatePlaceholder`), which the R2 design leaves field-less until R4
/// defines the real pipeline/bind-group cache — the write has no field to
/// land on yet.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:822-865`
pub fn GL_SetDefaultState(
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    assets: &RenderAssets,
    state: &mut TrImageState,
    gpu: &mut GpuResources,
) {
    // DEFERRED: R4 — qglClearDepth(1.0f); qglCullFace(GL_FRONT);
    // qglColor4f(1,1,1,1). Fixed-function GL surface, no R3 home (DEC-01/
    // DEC-37 A13.2).
    // Source: oracle/codemp/renderer/tr_init.cpp:824-828

    // DEFERRED: R4 — the `qglActiveTextureARB` non-null multitexture-
    // support gate has no R3 home (STATE HOMES: "declared outside the
    // renderer TU set… already homed by the engine port… confirm the exact
    // receiver at port time"); its guarded block
    // (`GL_SelectTexture(1)`/`GL_TextureMode`/`GL_TexEnv(GL_MODULATE)`/
    // `qglDisable(GL_TEXTURE_2D)`/`GL_SelectTexture(0)`) is skipped whole
    // rather than guessed at (porting-rules §A2) — the unconditional
    // `GL_TextureMode`/`GL_TexEnv` calls below it still run.
    // Source: oracle/codemp/renderer/tr_init.cpp:832-838

    // DEFERRED: R4 — qglEnable(GL_TEXTURE_2D). Fixed-function GL surface.
    // Source: oracle/codemp/renderer/tr_init.cpp:840

    let texture_mode = view.common.cvar(cvars.r_textureMode).string.clone();
    GL_TextureMode(view, cvars, assets, state, gpu, &texture_mode);
    GL_TexEnv(gpu, GL_MODULATE as u32);

    // DEFERRED: R4 — qglShadeModel(GL_SMOOTH); qglDepthFunc(GL_LEQUAL);
    // qglEnableClientState(GL_VERTEX_ARRAY). Fixed-function GL surface, no
    // R3 home. (`qglShadeModel` is absent from this wave's own GL/WGL
    // entry-point digest list — a digest gap, not a different disposition:
    // it is still the same `qgl*` surface.)
    // Source: oracle/codemp/renderer/tr_init.cpp:844-849

    // `glState.glStateBits = GLS_DEPTHTEST_DISABLE | GLS_DEPTHMASK_TRUE;` —
    // DEFERRED: `GpuResources::gl_state` (`GlStatePlaceholder`) carries no
    // fields yet (R2 leaves the pipeline/bind-group cache to R4); nothing
    // to write to. The `GLS_*` bit-flag `#define`s are also not yet ported
    // to Rust consts (same gap `GL_State`'s doc comment in `tr_backend.rs`
    // already flags).
    // Source: oracle/codemp/renderer/tr_init.cpp:854

    // DEFERRED: R4 — qglPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
    // qglDepthMask(GL_TRUE); qglDisable(GL_DEPTH_TEST);
    // qglEnable(GL_SCISSOR_TEST); qglDisable(GL_CULL_FACE);
    // qglDisable(GL_BLEND). Fixed-function GL surface, no R3 home.
    // Source: oracle/codemp/renderer/tr_init.cpp:856-861

    // `#ifdef _XBOX qglDisable( GL_LIGHTING ) #endif` dropped — MP retail
    // builds the non-`_XBOX` branch (established precedent, `R_Register`
    // doc comment above).
    // Source: oracle/codemp/renderer/tr_init.cpp:862-864
}

/// Raven `R_Splash`.
///
/// `#ifndef _XBOX` resolves to the always-taken branch — MP retail builds
/// the non-`_XBOX` configuration (established precedent, `R_Register` doc
/// comment above). The CPU-side setup (image lookup, the `GL_Bind`/
/// `GL_State`/`RB_SetGL2D` calls, the `if (pImage)` guard) is ported per
/// this wave's threading digest ("port the CPU logic"); two gaps are left
/// cited rather than guessed:
/// - The `qglBegin(GL_TRIANGLE_STRIP)`/`qglTexCoord2f`/`qglVertex2f`×4/
///   `qglEnd()` quad draw is the fixed-function GL surface DEC-01/DEC-37
///   leave unhomed until the R4 wgpu rewrite (A13.2) — the `width`/`height`/
///   `x1`/`x2`/`y1`/`y2` geometry feeds only this deferred draw call, so it
///   is described here rather than materialized as dead locals (the
///   `RB_Hyperspace`/`GL_SetDefaultState` precedent, `tr_backend.rs`/this
///   file above): `width=640, height=480, x1=320-width/2, x2=320+width/2,
///   y1=240-height/2, y2=240+height/2`, drawn as a texcoord-mapped
///   triangle strip covering that quad.
/// - `GLimp_EndFrame` — this packet's RESOLVED CALL SURFACE lists it as
///   already-ported LAW, but `crates/mp/renderer/Cargo.toml` has no
///   `mp_engine_client` dependency: the same "no reachable path from this
///   crate" gap `RB_SwapBuffers` already escalated
///   (`tr_backend.rs:812-816`) — a wave-planning discrepancy, raised here
///   rather than silently adding an undeclared cross-crate edge out of this
///   packet's scope.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:325-369`
pub fn R_Splash(
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    sim: &mut RenderAssetsSim,
    models: &RenderModels,
    state: &mut TrImageState,
    gpu: &mut GpuResources,
    frame: &mut FrameState,
    assets: &RenderAssets,
) {
    let p_image = R_FindImageFile(
        view,
        cvars,
        sim,
        models,
        state,
        gpu,
        Some("menu/splash"),
        false,
        false,
        false,
        GL_CLAMP,
    );

    RB_SetGL2D(frame, gpu, assets);
    if p_image.is_some() {
        // invalid paths?
        GL_Bind(gpu, p_image);
    }
    GL_State(gpu, GLS_SRCBLEND_ONE as u32 | GLS_DSTBLEND_ZERO as u32);

    // DEFERRED: R4 — qglBegin(GL_TRIANGLE_STRIP) / qglTexCoord2f /
    // qglVertex2f x4 / qglEnd() (see doc comment above). Fixed-function GL
    // surface, no R3 home (DEC-01/DEC-37 A13.2).
    // Source: oracle/codemp/renderer/tr_init.cpp:356-365

    // DEFERRED: GLimp_EndFrame — no reachable path from this crate (see doc
    // comment above).
    // Source: oracle/codemp/renderer/tr_init.cpp:367
}

/// Raven `InitOpenGL`.
///
/// `glConfig.vidWidth == 0` (STATE HOMES: `RenderAssets::glconfig`, R2
/// `## State ownership` `glConfig` row, `R2-D1`/B11 — sim-readable, not
/// render-thread-local) selects the first-init branch. `GLimp_Init` has no
/// reachable path from this crate today: it lives in `mp_engine_client::
/// null::null_glimp`, but `crates/mp/renderer/Cargo.toml` has no
/// `mp_engine_client` dependency — the same gap `GLimp_EndFrame`/
/// `GLimp_LogComment` already escalated (`tr_backend.rs:812-816`, `R_Splash`
/// doc comment above); escalated here rather than adding an undeclared
/// cross-crate edge out of this packet's scope. The rest of the branch (
/// `GL_SetDefaultState`/`R_Splash`/`GfxInfo_f`) is real CPU logic with a
/// resolved call surface and runs unconditionally either way.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:379-407`
#[allow(clippy::too_many_arguments)]
pub fn InitOpenGL(
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    assets: &RenderAssets,
    state: &mut TrImageState,
    gpu: &mut GpuResources,
    sim: &mut RenderAssetsSim,
    models: &RenderModels,
    frame: &mut FrameState,
) {
    if assets.glconfig.vid_width == 0 {
        // DEFERRED: GLimp_Init — no reachable path from this crate (see doc
        // comment above).
        // Source: oracle/codemp/renderer/tr_init.cpp:394

        // print info the first time only
        GL_SetDefaultState(view, cvars, assets, state, gpu);
        R_Splash(view, cvars, sim, models, state, gpu, frame, assets); // get something on screen asap
        GfxInfo_f(view, cvars, assets);
    } else {
        // set default state
        GL_SetDefaultState(view, cvars, assets, state, gpu);
    }
    // init command buffers and SMP
    r_init_command_buffers();
}

/// Raven `RE_Shutdown`.
///
/// The glow-teardown block (`r_DynamicGlow->integer` gate,
/// `qglDeleteProgramsARB`/`qglDeleteLists`/`qglCombinerParameteriNV`/
/// `qglDeleteTextures` x3 against `tr.glowVShader`/`glowPShader`/
/// `screenGlow`/`sceneImage`/`blurImage`) is entirely the fixed-function GL
/// surface DEC-01/DEC-37 leave unhomed until the R4 wgpu rewrite (A13.2) —
/// the glow handle fields themselves have no `RenderAssets`/`FrameState`
/// carrier either (STATE HOMES only assigns this wave `r_DynamicGlow`/`tr`),
/// so the whole guarded block is left as a single cited `// DEFERRED: R4`
/// rather than a partial stub, matching `GL_SetDefaultState`'s treatment of
/// its own `qgl*`-gated blocks above.
///
/// `R_TerrainShutdown`'s already-ported (wave 0) signature is `fn(cm: &mut
/// CollisionWorld, land_scape: &mut srfTerrain_t)`: `CollisionWorld` is
/// reachable (`view.cm`), but `tr.landScape` (`srfTerrain_t`) is
/// design-assigned to `RenderWorld::frame: FrameState`'s frontend-scratch
/// bucket (`renderer-r2-design.md` `## Seam definition`: "...sun/fog
/// fields, `landScape`, `distanceCull`...") and is not yet a landed
/// `FrameState` field — the same class of gap this file's
/// `tr.overbrightBits` notes already flag (`GfxInfo_f`/`R_TakeScreenshot`/
/// `R_TakeScreenshotJPEG` above). The call is deferred rather than the
/// carrier invented (preamble: "do NOT create a field").
///
/// `GLimp_Shutdown` has no reachable path from this crate:
/// `crates/mp/renderer/Cargo.toml` has no `mp_engine_client` dependency,
/// the same gap `GLimp_Init`/`GLimp_EndFrame` already escalated
/// (`R_Splash`/`InitOpenGL` doc comments above, `tr_backend.rs:812-816`) —
/// not added as an undeclared cross-crate edge out of this packet's scope.
///
/// Every other statement is real CPU logic with a resolved call surface and
/// lands here: the unconditional command removals, `R_ShutdownFonts`, the
/// `tr.registered`-gated `R_SyncRenderThread`/`R_ShutdownCommandBuffers`/
/// `R_DeleteTextures` sequence, and the final `tr.registered = qfalse`.
/// `qboolean destroyWindow` -> `bool` (translation dictionary).
///
/// Everything from the glow teardown through `tr.registered = qfalse` sits
/// inside one `#ifndef DEDICATED` (`:1349-1404`); the non-DEDICATED (client)
/// leg is transcribed per the R3 client-leg ruling — the R3 renderer track is
/// the CLIENT port, and the jampDed disposition is scoped to the
/// dedicated-server link set.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:1333-1407`
#[allow(clippy::too_many_arguments)]
pub fn RE_Shutdown(
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    assets: &mut RenderAssets,
    sim: &mut RenderAssetsSim,
    state: &mut TrImageState,
    gpu: &mut GpuResources,
    font: &mut FontState,
    destroy_window: bool,
) {
    Cmd_RemoveCommand(view.common, "imagelist");
    Cmd_RemoveCommand(view.common, "shaderlist");
    Cmd_RemoveCommand(view.common, "skinlist");
    Cmd_RemoveCommand(view.common, "screenshot");
    Cmd_RemoveCommand(view.common, "screenshot_tga");
    Cmd_RemoveCommand(view.common, "gfxinfo");
    Cmd_RemoveCommand(view.common, "r_atihack");
    Cmd_RemoveCommand(view.common, "r_we");
    Cmd_RemoveCommand(view.common, "imagecacheinfo");
    Cmd_RemoveCommand(view.common, "modellist");
    Cmd_RemoveCommand(view.common, "modelist");
    Cmd_RemoveCommand(view.common, "modelcacheinfo");

    let r_dynamic_glow = view.common.cvar(cvars.r_DynamicGlow).integer;
    if r_dynamic_glow != 0 {
        // Release the Glow Vertex Shader.
        // Release Pixel Shader.
        // Release the scene glow texture / scene texture / blur texture.
        //
        // DEFERRED: R4 — qglDeleteProgramsARB / qglDeleteLists /
        // qglCombinerParameteriNV / qglDeleteTextures x3 against
        // tr.glowVShader/glowPShader/screenGlow/sceneImage/blurImage (see
        // doc comment above). Fixed-function GL surface, no R3 home
        // (DEC-01/DEC-37 A13.2); the glow handle fields themselves have no
        // carrier either.
        // Source: oracle/codemp/renderer/tr_init.cpp:1354-1383
    }

    // DEFERRED: R_TerrainShutdown(&mut *view.cm, &mut tr.landScape) —
    // `tr.landScape` (`srfTerrain_t`) has no landed `FrameState` field yet
    // (see doc comment above); `CollisionWorld` alone (`view.cm`) is not
    // enough to make the call.
    // Source: oracle/codemp/renderer/tr_init.cpp:1386

    R_ShutdownFonts(font);

    if assets.registered {
        R_SyncRenderThread(assets, view.common, cvars);
        r_shutdown_command_buffers();
        if destroy_window {
            // only do this for vid_restart now, not during things like map
            // load
            R_DeleteTextures(sim, state, gpu);
        }
    }

    // shut down platform specific OpenGL stuff
    //
    // DEFERRED: GLimp_Shutdown() — no reachable path from this crate (see
    // doc comment above).
    // Source: oracle/codemp/renderer/tr_init.cpp:1400-1403

    assets.registered = false;
}

/// Raven `RE_EndRegistration`.
///
/// `#ifndef _XBOX` resolves to the always-taken branch — MP retail builds
/// the non-`_XBOX` configuration (established precedent, `R_Register`/
/// `R_Splash` doc comments above).
///
/// The whole fn sits inside one `#ifndef DEDICATED` (`:1409-1452`); it is
/// ported per the R3 client-leg ruling — the R3 renderer track is the CLIENT
/// port, and the jampDed disposition is scoped to the dedicated-server link
/// set.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:1418-1425`
pub fn RE_EndRegistration(
    common: &Common,
    cvars: &RendererCvars,
    assets: &RenderAssets,
    frame: &mut FrameState,
    gpu: &mut GpuResources,
) {
    R_SyncRenderThread(assets, common, cvars);
    if Sys_LowPhysicalMemory() == 0 {
        RB_ShowImages(frame, gpu, assets, cvars);
    }
}

/// Raven `#define MAX_POLYS 600` — corroborated by `RenderAssets::max_polys`'s
/// own doc comment ("default MAX_POLYS = 600").
///
/// Source: `oracle/codemp/renderer/tr_local.h:2256`
const MAX_POLYS: i32 = 600;

/// Raven `#define MAX_POLYVERTS 3000` — corroborated by
/// `RenderAssets::max_polyverts`'s own doc comment ("default MAX_POLYVERTS =
/// 3000").
///
/// Source: `oracle/codemp/renderer/tr_local.h:2257`
const MAX_POLYVERTS: i32 = 3000;

/// Raven `R_Init`.
///
/// `#ifdef _XBOX` blocks (the `externalVisData` save/restore straddling the
/// `tr`/`backEnd` memsets) drop — MP retail builds the non-`_XBOX` branch
/// (established precedent, `R_Register`/`R_Splash` doc comments above).
///
/// The three `#ifndef DEDICATED` legs — `R_InitFogTable`/`R_NoiseInit`
/// (`:1278-1282`), `R_ToggleSmpFrame` through `R_InitFonts` (`:1297-1313`),
/// and `R_InitDecals`/`R_InitWorldEffects`/`qglGetError` (`:1316-1324`) —
/// are all transcribed as the non-DEDICATED (client) leg per the R3
/// client-leg ruling: the R3 renderer track is the CLIENT port, and the
/// jampDed disposition is scoped to the dedicated-server link set.
///
/// `Com_Memset(&tr, 0, sizeof(tr))` / `Com_Memset(&backEnd, 0,
/// sizeof(backEnd))` — SPLIT per the STATE HOMES table. `backEnd`'s R2 home,
/// `FrameState`, also carries the `tr` frontend-scratch fields R2 folds into
/// it (`## Seam definition`), so both memsets land as one full-struct
/// rebuild below (nothing else in this fn's body writes into `frame`, so a
/// wholesale zero is exact, not approximate). `tr`'s registries land on
/// `RenderAssets` instead, and only get a **partial** reset here — the
/// residual fields this fn's own later statements don't already
/// re-establish:
/// - `defaultImage`/`fogImage`/`dlightImage`/`whiteImage`/`lightmaps`
///   (`tr_local.h:1329-1364`), `world`/`bspModels` (`:1039,1399`),
///   `distanceCull`/`distanceCullSquared` (`:1420`), `registered` (`:1310`),
///   `worldMapLoaded` (`:1320`) — no later statement in this fn touches
///   these, so they are zeroed here. `worldMapLoaded` in particular has to
///   be cleared: `RE_LoadWorldMap_Actual` (`tr_bsp.rs`) raises a redundant
///   -load `Com_Error` off it, so a `vid_restart` that left it set would fire
///   on the next map load.
/// - `sortedShaders` (`:1407`) — `CreateInternalShaders`
///   (`tr_shader.rs`, reached through `R_InitShaders` below) clears it, so a
///   redundant pre-clear here is skipped.
/// - `shaders` (`:1405-1406`) — rebuilt by `R_InitShaders` below, which
///   clears `shader_lookup`/`defer_load` and hands the arena purge itself to
///   `CreateInternalShaders`' `Arena::reset` (DEC-42.1: every pre-reset
///   handle stales, the `<default>` shader is re-created at index 0), so a
///   pre-clear here would be redundant.
/// - `skins` (`:1409-1410`) — rebuilt by `R_InitSkins` below, which hands the
///   arena purge to `Arena::reset` (DEC-42.1: every pre-reset handle stales,
///   the `"<default skin>"` entry is re-created at index 0), so a pre-clear
///   here would be redundant.
/// - `models` (`:1396-1397`) — rebuilt by `RenderModels::model_init` below,
///   which hands the pool purge to `ModelPool::reset` (DEC-42.1: the
///   high-water mark drops, every pre-reset handle above slot 0 stales) and
///   re-creates the `MOD_BAD` entry at index 0, so a pre-clear here would be
///   redundant. The pool lives on `RenderModels`, not `RenderAssets`
///   (`docs/subsystems/tr-model.md` amendment 2026-07-27, #51).
/// - `skin_lookup` — a plain name->handle map with no reserved-slot
///   semantics; zeroed here (`R_InitSkins` clears it again as the map half of
///   its own purge).
/// - `images`/`image_names` (`AllocatedImages`, a separate `tr_image.cpp`
///   global, not a `trGlobals_t` field), `shader_text`/
///   `shader_text_hash_table`/`defer_load`'s own storage (`s_shaderText`/
///   `shaderTextHashTable`, separate `tr_shader.cpp` file-scope statics,
///   A13.4), and `glConfig` ("outside of TR… shouldn't be cleared during ref
///   re-init", `## Seam definition`) are never part of `tr` at all — left
///   untouched, matching Raven.
/// - `function_tables`/`max_polys`/`max_polyverts` are unconditionally
///   overwritten by this same fn's own next statements (the function-table
///   loop, `R_InitFogTable`, the `max_polys`/`max_polyverts` clamp below) —
///   a pre-zero would be immediately discarded, so it is skipped.
///
/// `#ifndef DEDICATED Com_Memset(&tess, 0, sizeof(tess)) #endif` and the
/// `#ifndef FINAL_BUILD` `tess.xyz` 16-byte-alignment warning both need
/// `tess` (`shaderCommands_t`) — DISSOLVED into R4's tessellation pipeline,
/// no R3 carrier (`## State ownership` `tess` row); both are dropped rather
/// than guessed at.
///
/// `Hunk_Alloc(sizeof(*backEndData) + …)` / `backEndData = …` /
/// `backEndData->polys = …` / `backEndData->polyVerts = …` — `backEndData_t`
/// is DISSOLVED (`## Seam definition` A1 disposition table: "its field list
/// is the reference vocabulary for `FrameData`'s event payloads, not a
/// struct that survives"); the one durable value this block produces,
/// `max_polys`/`max_polyverts` as append-time capacity bounds, is already
/// captured on `RenderAssets` by the clamp logic immediately above it.
///
/// `RE_SetLightStyle(i, -1)` — packed `int color = -1` reinterpreted as
/// `color4ub_t` is `[0xFF; 4]` regardless of host byte order (all bits set).
///
/// `G2VertSpaceServer = &CMiniHeap_singleton;` — dropped, not given a state
/// home: `CMiniHeap` is deleted per the ghoul2-server design ruling already
/// applied to this exact assignment's server-side counterpart
/// (`SV_SpawnServer`'s `G2VertSpaceServer = new CMiniHeap(...)`,
/// `crates/mp/engine/server/src/sv_init.rs:1172-1175`: "`CMiniHeap` is
/// deleted per the ghoul2-server design (the collision path threads no
/// scratch heap), so this allocation drops"); the client-side singleton
/// assignment is the same dead surface (porting-rules §20).
///
/// `R_TerrainInit()` — its already-ported (wave 0) signature needs
/// `land_scape: &mut srfTerrain_t` (`tr.landScape`), which is
/// design-assigned to `FrameState`'s frontend-scratch bucket but not yet a
/// landed field (the same gap `RE_Shutdown`'s `R_TerrainShutdown` doc
/// comment above already escalates for the same global) — deferred rather
/// than inventing the field.
///
/// `int err = qglGetError(); if (err != GL_NO_ERROR) Com_Printf(...)` — the
/// fixed-function GL surface DEC-01/DEC-37 leave unhomed until the R4 wgpu
/// rewrite (A13.2), same treatment as every other `qgl*` call in this file.
///
/// `assets: &mut RenderAssets` and `sim: &mut RenderAssetsSim` (whose own
/// `published: Arc<RenderAssets>` is a *separate* instance) are threaded as
/// independent sibling parameters, mirroring `R_InitShaders`'s own
/// already-ported (wave 9) signature, which carries the same duality.
/// Reconciling them into one Arc-published instance across a frame boundary
/// (A9) is engine call-site wiring outside this single-fn packet's scope.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:1214-1326`
#[allow(clippy::too_many_arguments)]
pub fn R_Init(
    view: &mut EngineHostView,
    cvars: &mut RendererCvars,
    assets: &mut RenderAssets,
    sim: &mut RenderAssetsSim,
    state: &mut TrImageState,
    gpu: &mut GpuResources,
    models: &mut RenderModels,
    frame: &mut FrameState,
    scene: &mut SceneState,
    frame_data: &mut FrameData,
    noise: &mut NoiseState,
    rng: &mut Rng,
    font: &mut FontState,
    world_effects: &mut WorldEffectsState,
    qs: &mut QSharedScratch,
    sky_view: &mut viewParms_t,
    sky: &mut SkyState,
) {
    // Com_Memset(&tr, 0, sizeof(tr)) — partial reset; see doc comment above
    // for the residual-field reasoning.
    assets.default_image = None;
    assets.fog_image = None;
    assets.dlight_image = None;
    assets.white_image = None;
    assets.lightmaps.clear();
    assets.world = None;
    assets.bsp_models.clear();
    assets.distance_cull = 0.0;
    assets.distance_cull_squared = 0.0;
    assets.registered = false;
    assets.world_map_loaded = false;
    assets.skin_lookup.clear();

    // `RenderAssets::shaders` is purged by `R_InitShaders` ->
    // `CreateInternalShaders`' `Arena::reset` below (DEC-42.1), so it is not
    // touched here.

    // `RenderAssets::skins` is purged by `R_InitSkins`' `Arena::reset` below
    // (DEC-42.1), so it is not touched here either.

    // `tr.models`/`tr.numModels`/`mhHashTable` (`tr_local.h:1396-1397`) — the
    // memset's model half. Nothing to pre-clear here: the pool lives on
    // `RenderModels` (`tr_model/model_pool.rs`, amendment 2026-07-27 / #51),
    // and `models.model_init()` below IS its DEC-42.1 reset — it drops the
    // high-water mark, stales every pre-reset handle above slot 0, clears the
    // hash, and re-creates the `MOD_BAD` entry at slot 0 (`R_ModelInit`,
    // `oracle/codemp/renderer/tr_model.cpp:1665-1680`). Same disposition as
    // `shaders`/`skins` above, whose purges also ride their own `R_Init*`
    // rebuild statement.

    // Com_Memset(&backEnd, 0, sizeof(backEnd)) — full rebuild; see doc
    // comment above (nothing else in this fn writes into `frame`).
    *frame = FrameState {
        refdef: TrRefdef::default(),
        view: ViewParms::default(),
        ori: OrientationR::default(),
        counters: BackEndCounters {},
        is_hyperspace: false,
        current_entity: None,
        sky_rendered_this_view: false,
        projection_2d: false,
        color_2d: [0; 4],
        vertexes_2d: false,
        entity_2d: RefEntity::default(),
        scene_light_styles: [[0; 4]; MAX_LIGHT_STYLES],
        frame_count: 0,
        view_count: 0,
        scene_count: 0,
        frame_scene_num: 0,
        vis_count: 0,
        view_cluster: 0,
        skyboxportal: 0,
        drawskyboxportal: 0,
        render_glowing_objects: false,
        identity_light: 0.0,
        identity_light_byte: 0,
        overbright_bits: 0,
        sun_direction: [0.0; 3],
        sun_ambient: [0.0; 3],
        external_vis_data: None,
    };

    // DEFERRED: `tess` (`shaderCommands_t`) memset + the `tess.xyz` 16-byte
    // alignment warning — `tess` is DISSOLVED into R4's tessellation
    // pipeline, no R3 carrier (see doc comment above).
    // Source: oracle/codemp/renderer/tr_init.cpp:1235,1246-1250

    //
    // init function tables
    //
    for i in 0..FUNCTABLE_SIZE {
        let deg = i as f32 * 360.0 / (FUNCTABLE_SIZE - 1) as f32;
        // DEG2RAD's `a * M_PI` promotes through `M_PI` (double); ruling 12.
        let rad = deg as f64 * PI / 180.0;
        assets.function_tables.sin_table[i] = rad.sin() as f32;
        assets.function_tables.square_table[i] = if i < FUNCTABLE_SIZE / 2 { 1.0 } else { -1.0 };
        assets.function_tables.saw_tooth_table[i] = i as f32 / FUNCTABLE_SIZE as f32;
        assets.function_tables.inverse_saw_tooth_table[i] =
            1.0 - assets.function_tables.saw_tooth_table[i];

        if i < FUNCTABLE_SIZE / 2 {
            if i < FUNCTABLE_SIZE / 4 {
                assets.function_tables.triangle_table[i] = i as f32 / (FUNCTABLE_SIZE / 4) as f32;
            } else {
                assets.function_tables.triangle_table[i] =
                    1.0 - assets.function_tables.triangle_table[i - FUNCTABLE_SIZE / 4];
            }
        } else {
            assets.function_tables.triangle_table[i] =
                -assets.function_tables.triangle_table[i - FUNCTABLE_SIZE / 2];
        }
    }

    R_InitFogTable(assets);
    // `R_CreateFogImage` reads the fog table through `sim.published` (the A9
    // duality), so the table mirrors across, as the four internal image
    // handles do below. Without this the baked fog image has zero alpha in
    // every texel.
    Arc::make_mut(&mut sim.published).function_tables.fog_table =
        assets.function_tables.fog_table;

    R_NoiseInit(noise, rng);

    R_Register(view, cvars);

    let mut max_polys = view.common.cvar(cvars.r_maxpolys).integer;
    if max_polys < MAX_POLYS {
        max_polys = MAX_POLYS;
    }
    assets.max_polys = max_polys as usize;

    let mut max_polyverts = view.common.cvar(cvars.r_maxpolyverts).integer;
    if max_polyverts < MAX_POLYVERTS {
        max_polyverts = MAX_POLYVERTS;
    }
    assets.max_polyverts = max_polyverts as usize;

    // DEFERRED: `ptr = Hunk_Alloc(sizeof(*backEndData) + …); backEndData =
    // …; backEndData->polys = …; backEndData->polyVerts = …;` —
    // `backEndData_t` is DISSOLVED (see doc comment above); the durable
    // `max_polys`/`max_polyverts` capacity bounds are already set above.
    // Source: oracle/codemp/renderer/tr_init.cpp:1293-1296

    R_ToggleSmpFrame(frame_data, scene);

    for i in 0..MAX_LIGHT_STYLES {
        RE_SetLightStyle(sim, i, [0xFF; 4]);
    }

    InitOpenGL(view, cvars, &*assets, state, gpu, sim, &*models, frame);

    R_InitImages(
        view,
        cvars,
        &assets.glconfig,
        sim,
        &*models,
        state,
        gpu,
        &mut *frame,
    );
    // PORT-NOTE: Raven has one `tr`, so `R_InitImages`' internal-image handles
    // (`tr.defaultImage`/`whiteImage`/`fogImage`/`dlightImage`) are already
    // visible to `R_InitShaders` when it builds `<default>`/`white`/`fog`.
    // Here the image registry is the sim-published master (A9) and the shader
    // registry is `assets`, so the four handles are mirrored across before the
    // shader init reads them — without this `CreateInternalShaders` binds
    // `tr.defaultImage == NULL` and every internal shader loses its only stage.
    // The handles stay valid on either side: both name slots in the one image
    // arena `R_CreateImage` writes.
    assets.default_image = sim.published.default_image;
    assets.fog_image = sim.published.fog_image;
    assets.dlight_image = sim.published.dlight_image;
    assets.white_image = sim.published.white_image;

    R_InitShaders(
        false, qs, frame, assets, view, cvars, sim, &*models, state, gpu, sky_view, sky,
    );
    // R_InitSkins(): the client registry (`RenderAssets::skins`) and, for the
    // dedicated link set's own `RenderModels.skins` pool, its twin — one
    // oracle fn per registry (`tr_image.rs`'s skin PORT-NOTE).
    R_InitSkins(assets);
    models.init_skins();

    // DEFERRED: `R_TerrainInit()` — its already-ported signature needs
    // `land_scape: &mut srfTerrain_t` (`tr.landScape`), not yet a landed
    // `FrameState` field (see doc comment above).
    // Source: oracle/codemp/renderer/tr_init.cpp:1310

    R_InitFonts(font);

    models.model_init();

    // PORT-NOTE: `G2VertSpaceServer = &CMiniHeap_singleton;` dropped — see
    // doc comment above (established `sv_init.rs` ghoul2-server precedent).
    // Source: oracle/codemp/renderer/tr_init.cpp:1315

    R_InitDecals(scene);

    world_effects.R_InitWorldEffects(view);

    // DEFERRED: R4 — `int err = qglGetError(); if (err != GL_NO_ERROR)
    // Com_Printf(...)`. Fixed-function GL surface, no R3 home (DEC-01/
    // DEC-37 A13.2).
    // Source: oracle/codemp/renderer/tr_init.cpp:1321-1323
}
