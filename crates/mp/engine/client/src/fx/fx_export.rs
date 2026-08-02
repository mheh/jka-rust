//! Raven `FXExport.cpp` — the entry points the `CG_FX_*` trap arms call.
//!
//! Each one refreshes the refdef and cvar snapshot first, because Raven read the
//! module's live `cg.refdef` pointer and the cvar structs on every access.
//!
//! Source: `oracle/codemp/client/FXExport.cpp`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use native_math::vector::vec3_t;

use crate::fx::fx_host::FxHost;
use crate::fx::fx_scheduler::{
    fx_add_scheduled_effects, fx_draw_2d_effects, fx_play_effect_axis, fx_play_effect_file_fwd,
    fx_play_effect_fwd, fx_register_effect,
};
use crate::fx::fx_system::FxSystem;
use crate::fx::fx_util::{FX_Free, FX_Init, FX_SetRefDef};

/// Raven `FX_RegisterEffect`.
///
/// Source: `oracle/codemp/client/FXExport.cpp:13-16`
pub fn FX_RegisterEffect(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, file: &str) -> c_int {
    fx.refresh(host);
    fx_register_effect(fx, host, file)
}

/// Raven `FX_PlayEffect` — play by file name with a forward vector.
///
/// Source: `oracle/codemp/client/FXExport.cpp:18-36`
pub fn FX_PlayEffect(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    file: &str,
    org: vec3_t,
    fwd: vec3_t,
    vol: c_int,
    rad: c_int,
) {
    fx.refresh(host);
    fx_play_effect_file_fwd(fx, host, file, org, fwd, vol, rad);
}

/// Raven `FX_PlayEffectID`.
///
/// Source: `oracle/codemp/client/FXExport.cpp:38-56`
#[allow(clippy::too_many_arguments)]
pub fn FX_PlayEffectID(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    id: c_int,
    org: vec3_t,
    fwd: vec3_t,
    vol: c_int,
    rad: c_int,
    is_portal: bool,
) {
    fx.refresh(host);
    fx_play_effect_fwd(fx, host, id, org, fwd, vol, rad, is_portal);
}

/// Raven `FX_PlayBoltedEffectID` — no axis, the bolt supplies one.
///
/// Source: `oracle/codemp/client/FXExport.cpp:58-62`
#[allow(clippy::too_many_arguments)]
pub fn FX_PlayBoltedEffectID(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    id: c_int,
    org: vec3_t,
    bolt_info: c_int,
    i_ghoul2: c_int,
    i_loop_time: c_int,
    is_relative: bool,
) {
    fx.refresh(host);
    // Raven passes a null axis here. A bolted effect always schedules, and the
    // scheduled path never reads the axis, so the zero axis below is never used.
    fx_play_effect_axis(
        fx,
        host,
        id,
        Some(org),
        [[0.0; 3]; 3],
        bolt_info,
        i_ghoul2,
        -1,
        -1,
        -1,
        false,
        i_loop_time,
        is_relative,
    );
}

/// Raven `FX_PlayEntityEffectID`.
///
/// Source: `oracle/codemp/client/FXExport.cpp:64-75`
#[allow(clippy::too_many_arguments)]
pub fn FX_PlayEntityEffectID(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    id: c_int,
    org: vec3_t,
    axis: [vec3_t; 3],
    bolt_info: c_int,
    _ent_num: c_int,
    vol: c_int,
    rad: c_int,
) {
    fx.refresh(host);
    // Raven drops `entNum` here and passes NULL for the ghoul2 handle.
    fx_play_effect_axis(
        fx,
        host,
        id,
        Some(org),
        axis,
        bolt_info,
        0,
        -1,
        vol,
        rad,
        false,
        0,
        false,
    );
}

/// Raven `FX_AddScheduledEffects`.
///
/// Source: `oracle/codemp/client/FXExport.cpp:77-80`
pub fn FX_AddScheduledEffects(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, portal: bool) {
    fx.refresh(host);
    fx_add_scheduled_effects(fx, host, portal);
}

/// Raven `FX_Draw2DEffects`.
///
/// Source: `oracle/codemp/client/FXExport.cpp:82-85`
pub fn FX_Draw2DEffects(
    fx: &mut FxSystem,
    host: &mut FxHost<'_, '_>,
    screen_x_scale: f32,
    screen_y_scale: f32,
) {
    fx.refresh(host);
    fx_draw_2d_effects(fx, host, screen_x_scale, screen_y_scale);
}

/// Raven `FX_InitSystem`.
///
/// Source: `oracle/codemp/client/FXExport.cpp:87-90`
pub fn FX_InitSystem(fx: &mut FxSystem, host: &mut FxHost<'_, '_>, refdef: *mut refdef_t) -> c_int {
    FX_Init(fx, host, refdef)
}

/// Raven `FX_SetRefDefFromCGame`.
///
/// Source: `oracle/codemp/client/FXExport.cpp:92-95`
pub fn FX_SetRefDefFromCGame(fx: &mut FxSystem, refdef: *mut refdef_t) {
    FX_SetRefDef(fx, refdef);
}

/// Raven `FX_FreeSystem`.
///
/// Source: `oracle/codemp/client/FXExport.cpp:97-100`
pub fn FX_FreeSystem(fx: &mut FxSystem, host: &mut FxHost<'_, '_>) -> c_int {
    FX_Free(fx, host, true) as c_int
}

/// Raven `FX_AdjustTime`.
///
/// Source: `oracle/codemp/client/FXExport.cpp:102-105`
pub fn FX_AdjustTime(fx: &mut FxSystem, time: c_int) {
    fx.clock.AdjustTime(time);
}
