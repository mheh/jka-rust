//! Raven `tr_init.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_init.cpp`

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::cvar_fns::{Cvar_Get, Cvar_Set, Cvar_VariableString};
use mp_engine_qcommon::files_common::FS_WriteFile;
use mp_qshared::common::mp::cgame::texture_compression_t::textureCompression_t;
use mp_qshared::shared::cvar::{
    CvarHandle, CVAR_ARCHIVE, CVAR_CHEAT, CVAR_LATCH, CVAR_ROM, CVAR_TEMP,
};
use mp_qshared::shared::q_color::S_COLOR_YELLOW;
use native_platform::Sys_LowPhysicalMemory;

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
/// `r_atihack`/`r_we`/`imagecacheinfo`) is dropped whole — DEDICATED is this
/// build's live configuration (`Hunk_Clear` precedent,
/// `crates/mp/engine/qcommon/src/z_memman_pc.rs:808-811`, porting-rules
/// §20/§C10).
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

    // PORT-NOTE: the `#ifndef DEDICATED` command-registration block
    // (imagelist/shaderlist/skinlist/screenshot/screenshot_tga/gfxinfo/
    // r_atihack/r_we/imagecacheinfo) is dropped whole — DEDICATED is this
    // build's live configuration (see doc comment above).
    // Source: oracle/codemp/renderer/tr_init.cpp:1188-1197

    // TODO: Port R_Modellist_f
    // Source: oracle/codemp/renderer/tr_init.cpp:1199
    // Registered unconditionally (outside `#ifndef DEDICATED`) as
    // "modellist" — not yet ported anywhere in this crate.

    // TODO: Port R_ModeList_f Cmd_AddCommand wiring
    // Source: oracle/codemp/renderer/tr_init.cpp:1201
    // `R_ModeList_f` (this file — `common: &mut Common, vidmodes:
    // &VidModeTable`) is already ported but does not fit `CmdFunction =
    // fn(&mut EngineHostView)` (`crates/mp/engine/qcommon/src/cmd/
    // cmd_function_t.rs:12`) — no renderer-state-carrying adapter is
    // licensed by this packet's resolved call surface.

    // TODO: Port RE_RegisterModels_Info_f
    // Source: oracle/codemp/renderer/tr_init.cpp:1203
    // Registered unconditionally (outside `#ifndef DEDICATED`) as
    // "modelcacheinfo" — not yet ported anywhere in this crate.
}
