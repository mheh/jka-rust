//! Raven `tr_arioche.cpp` logic (R3 frontend port).
//!
//! Source: `oracle/codemp/renderer/tr_arioche.cpp`

// Raven-named functions/types keep their original casing across this
// transcription, matching the rest of the renderer crate.
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::cvar_fns::{Cvar_Set, Cvar_VariableStringBuffer};
use mp_qshared::shared::com_parse::QSharedScratch;
use mp_qshared::shared::MAX_QPATH;
use native_math::qmath::{ColorBytes4, Com_Clampi, NormalToLatLong};
use native_string::{atoi, buf_to_string};

use crate::render_state::frame_state::FrameState;
use crate::render_state::gpu_resources::GpuResources;
use crate::render_state::render_assets::RenderAssets;
use crate::render_state::render_assets_sim::RenderAssetsSim;
use crate::render_state::renderer_cvars::RendererCvars;
use crate::render_state::shader_asset::ShaderHandle;
use crate::tr_bsp::R_ColorShiftLightingBytes;
use crate::tr_image::TrImageState;
use crate::tr_local::view_parms_t::viewParms_t;
use crate::tr_model::render_models::RenderModels;
use crate::tr_shader::{lightmapsNone, stylesDefault, R_FindShader, R_RemapShader};
use crate::tr_sky::SkyState;
use crate::tr_worldeffects::world_effects::WorldEffectsState;

/// Raven `R_RMGInit` — RMG (dynamic-terrain) level-load hook: remaps the sky
/// shader, seeds the light-grid's first sample from the sun direction, and
/// dispatches the `RMG_weather` cvar's rain/snow world effect.
///
/// STATE HOMES (this packet's row): `tr` is SPLIT per R2 `## State
/// ownership` — the registries this fn touches (`world`) live on
/// `RenderAssets`, the frontend scratch it reads (`sunDirection`) on
/// `FrameState`.
///
/// One section is genuinely DEFERRED (never-guess rule, porting-rules §A2 /
/// packet marker law rule 12), cited at its own site below:
/// - `grid->directLight[0]` of the light-grid byte fill (`:57-60`) —
///   `tr.sunLight` has no R2 carrier (`tr_shader.rs:4489-4493`'s doc comment:
///   "`## State ownership` names `tr.sunDirection`'s home ... but not
///   [`sunLight`]").
///
/// Two more sections were previously deferred on premises this wave's field
/// merges have since closed and are now landed:
/// - `sky = R_FindShader(newSky, lightmapsNone, stylesDefault, qfalse)`
///   (`:26`) is still *omitted*, but only because `sky`'s value is never read
///   again in this fn (oracle `:26-29`) — `lightmapsNone`/`stylesDefault`
///   (`tr_shader.rs:172,196`) and `R_FindShader` (`tr_shader.rs:4839`) are in
///   fact already landed, so the omission is dead-code elision, not a
///   blocked read.
/// - `grid->ambientLight[0]` of the light-grid byte fill (`:52-55`) is landed
///   below, `tr.sunAmbient` now real (`FrameState::sun_ambient`).
/// - The global-fog override block (`:74-96`) is landed below — `WorldAsset`
///   now carries `global_fog`/`fogs` and `ShaderAsset` now carries
///   `fog_parms` (both cited at the site below).
///
/// `R_RemapShader` (called for real, below) is itself an already-landed
/// wave-9 loud stub (`tr_shader.rs:5169-5238`) — this call panics via that
/// stub until its owning wave fills it in (marker law: "Panics via
/// <callee>'s loud stub until its owning wave lands"). `R_FindShader` (called
/// for real in the global-fog block below) is a fully landed wave-7 fn, not a
/// stub.
///
/// Source: `oracle/codemp/renderer/tr_arioche.cpp:12-115`
#[allow(clippy::too_many_arguments)]
pub fn R_RMGInit(
    qs: &mut QSharedScratch,
    frame: &mut FrameState,
    assets: &mut RenderAssets,
    view: &mut EngineHostView,
    cvars: &RendererCvars,
    sim: &mut RenderAssetsSim,
    models: &RenderModels,
    img_state: &mut TrImageState,
    gpu: &mut GpuResources,
    sky_view: &mut viewParms_t,
    sky: &mut SkyState,
    world_effects: &mut WorldEffectsState,
) {
    // PORT-NOTE: `Cvar_VariableStringBuffer`'s LAW signature writes through a
    // `char[MAX_QPATH]` C buffer; bridged here via a local Latin-1 `[u8;
    // MAX_QPATH]` array (cast to `*mut c_char` only as a pointer value, never
    // dereferenced in this file) + `native_string::buf_to_string`'s NUL-
    // terminated-buffer decode, so no `unsafe`/`c_char` field is adopted by
    // this file's interior (interior-safety law) — the callee (already
    // ported, untouched here) owns the one unsafe deref.
    let mut new_sky_buf = [0u8; MAX_QPATH];
    Cvar_VariableStringBuffer(
        view.common,
        "RMG_sky",
        new_sky_buf.as_mut_ptr() as *mut c_char,
        MAX_QPATH as c_int,
    );
    let new_sky = buf_to_string(&new_sky_buf);

    // Get sunlight - this should set up all the sunlight data
    //
    // OMITTED (not blocked): `sky = R_FindShader(newSky, lightmapsNone,
    // stylesDefault, qfalse)` — see doc comment above. `sky`'s value is
    // never read again in this fn (oracle:26-29), so omitting the call
    // changes no other transcribed value; `R_FindShader`/`lightmapsNone`/
    // `stylesDefault` are all already landed (used for real in the
    // global-fog block below).
    // Source: oracle/codemp/renderer/tr_arioche.cpp:24-26

    // Remap sky
    R_RemapShader(
        "textures/tools/_sky",
        &new_sky,
        None,
        qs,
        frame,
        assets,
        view,
        cvars,
        sim,
        models,
        img_state,
        gpu,
        sky_view,
        sky,
    );

    // Fill in the lightgrid with sunlight
    if let Some(world) = assets.world.as_mut() {
        let has_light_grid = world.light_grid_data.is_some();
        if has_light_grid {
            if let Some(first) = world.light_grid_data.as_mut().and_then(|g| g.get_mut(0)) {
                // `grid->ambientLight[0]` (oracle:52-55) — Com_Clampi + the
                // color shift, now landed (`FrameState::sun_ambient`).
                //
                // `R_ColorShiftLightingBytes`'s 3-component overload
                // (`R_ColorShiftLightingBytesRGB`, `tr_bsp.rs:349`) is
                // private to `tr_bsp.rs`, out of this fixer's file scope; the
                // already-`pub` 4-component overload (`tr_bsp.rs:313`)
                // computes the identical r/g/b shift and passes its 4th byte
                // through unchanged (`tr_bsp.rs:338`), so it is reused here
                // with a throwaway alpha slot, reproducing the 3-component
                // result byte-for-byte.
                let ambient_in: [u8; 4] = [
                    Com_Clampi(0, 255, (frame.sun_ambient[0] * 255.0) as c_int) as u8,
                    Com_Clampi(0, 255, (frame.sun_ambient[1] * 255.0) as c_int) as u8,
                    Com_Clampi(0, 255, (frame.sun_ambient[2] * 255.0) as c_int) as u8,
                    0,
                ];
                let ambient_out = R_ColorShiftLightingBytes(frame, ambient_in);
                first.ambientLight[0] = [ambient_out[0], ambient_out[1], ambient_out[2]];

                // DEFERRED: `grid->directLight[0]` (oracle:57-60) —
                // `tr.sunLight` has no `FrameState` carrier (see doc comment
                // above).
                // Source: oracle/codemp/renderer/tr_arioche.cpp:57-60

                NormalToLatLong(frame.sun_direction, first.latLong.as_mut_ptr());
            }

            let n =
                (world.num_grid_array_elements.max(0) as usize).min(world.light_grid_array.len());
            world.light_grid_array[..n].fill(0);
        }
    }

    // Override the global fog with the defined one
    let global_fog_index = match assets.world.as_ref() {
        Some(w) if w.global_fog != -1 => Some(w.global_fog),
        _ => None,
    };
    if let Some(global_fog_index) = global_fog_index {
        let mut new_fog_buf = [0u8; MAX_QPATH];
        Cvar_VariableStringBuffer(
            view.common,
            "RMG_fog",
            new_fog_buf.as_mut_ptr() as *mut c_char,
            MAX_QPATH as c_int,
        );
        let new_fog = buf_to_string(&new_fog_buf);

        let fog = R_FindShader(
            &new_fog,
            &lightmapsNone,
            &stylesDefault,
            false,
            qs,
            frame,
            assets,
            view,
            cvars,
            sim,
            models,
            img_state,
            gpu,
            sky_view,
            sky,
        );

        if fog != ShaderHandle::slot_zero() {
            // DIVERGE (porting-rules §19): `gfog->parms = *fog->fogParms`
            // (oracle:81) dereferences the shader's `fogParms` pointer
            // unconditionally — a shader reached here with no `fogParms`
            // (never declared the `fog` shader keyword) is a null deref in
            // the oracle (UB). The defined substitute picked here is to skip
            // the write, the one defined behavior available.
            if let Some(fog_parms) = assets.shaders.get(fog).and_then(|s| s.fog_parms) {
                if let Some(world) = assets.world.as_mut() {
                    if let Some(gfog) = world.fogs.get_mut(global_fog_index as usize) {
                        gfog.parms = fog_parms;
                        if gfog.parms.depth_for_opaque != 0.0 {
                            gfog.tc_scale = 1.0 / (gfog.parms.depth_for_opaque * 8.0);
                            assets.distance_cull = gfog.parms.depth_for_opaque;
                            assets.distance_cull_squared =
                                assets.distance_cull * assets.distance_cull;
                            Cvar_Set(
                                view,
                                "RMG_distancecull",
                                &format!("{:.6}", assets.distance_cull),
                            );
                        } else {
                            gfog.tc_scale = 1.0;
                        }
                        gfog.color_int = ColorBytes4(
                            gfog.parms.color[0],
                            gfog.parms.color[1],
                            gfog.parms.color[2],
                            1.0,
                        );
                    }
                }
            }
        }
    }

    // Set up any weather effects
    let mut weather_buf = [0u8; MAX_QPATH];
    Cvar_VariableStringBuffer(
        view.common,
        "RMG_weather",
        weather_buf.as_mut_ptr() as *mut c_char,
        MAX_QPATH as c_int,
    );
    let weather = buf_to_string(&weather_buf);

    // PORT-NOTE: `atol` -> `native_string::atoi` (translation dictionary,
    // `atof`/`atoi` -> `native_string`, never `.parse()`); the switch below
    // only distinguishes 0/1/2, well within `atoi`'s `i32` result.
    match atoi(&weather) {
        0 => {}
        1 => {
            world_effects.R_WorldEffectCommand(
                qs,
                view,
                cvars,
                sim,
                models,
                img_state,
                gpu,
                Some(b"rain init 1000"),
            );
            world_effects.R_WorldEffectCommand(
                qs,
                view,
                cvars,
                sim,
                models,
                img_state,
                gpu,
                Some(b"rain outside"),
            );
        }
        2 => {
            world_effects.R_WorldEffectCommand(
                qs,
                view,
                cvars,
                sim,
                models,
                img_state,
                gpu,
                Some(b"snow init 1000 outside"),
            );
            world_effects.R_WorldEffectCommand(
                qs,
                view,
                cvars,
                sim,
                models,
                img_state,
                gpu,
                Some(b"snow outside"),
            );
        }
        _ => {}
    }
}
