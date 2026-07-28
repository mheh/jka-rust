//! Port of `oracle/codemp/cgame/cg_light.c` — dynamic light styles and their per-frame animation. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]

use core::ffi::c_int;

use mp_bg::public::configstring::CS_LIGHT_STYLES;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::MAX_QPATH;
use native_string::string_to_latin1;

use crate::cg_main::{CG_ConfigString, Com_Error};
use crate::trap;
use crate::world::CgContext;

/// Raven `#if !defined MAX_LIGHT_STYLES / #define MAX_LIGHT_STYLES 64` — the
/// configstring-driven lightstyle table's fixed size.
///
/// Source: `oracle/codemp/game/q_shared.h:423-424`
pub const MAX_LIGHT_STYLES: usize = 64;

/// Raven `CG_RunLightStyles` — recomputes every lightstyle's current color
/// from its compiled animation map and pushes it to the renderer.
///
/// Raven's `ofs == lastofs` early-out is commented out in the oracle (dead
/// code kept for reference), so every call walks and re-pushes the full
/// table even when the 50ms bucket hasn't advanced.
///
/// PORT-NOTE: `trap_R_SetLightStyle`'s color arg is Raven's raw
/// `*(int*)ls->value` pointer-cast of the 4-byte RGBA array; `i32::from_ne_bytes`
/// reproduces that same native-endian reinterpretation without a pointer cast.
///
/// Source: `oracle/codemp/cgame/cg_light.c:33-66`
pub fn CG_RunLightStyles(ctx: &mut CgContext) {
    let ofs = ctx.world.cg.time / 50;
    // if (ofs == lastofs) return; -- Raven left this early-out commented out (see doc above).
    ctx.world.light.lastofs = ofs;

    for i in 0..MAX_LIGHT_STYLES {
        let ls = &mut ctx.world.light.cl_lightstyle[i];
        if ls.length == 0 {
            ls.value[0] = 255;
            ls.value[1] = 255;
            ls.value[2] = 255;
            ls.value[3] = 255;
        } else if ls.length == 1 {
            ls.value[0] = ls.map[0][0];
            ls.value[1] = ls.map[0][1];
            ls.value[2] = ls.map[0][2];
            ls.value[3] = 255; // ls.map[0][3]
        } else {
            // C: ofs % ls->length, both plain `int` - keep the modulo in i32 before
            // indexing so a hypothetical negative `ofs` behaves like C's truncating
            // division rather than usize wraparound.
            let idx = (ofs % ls.length) as usize;
            ls.value[0] = ls.map[idx][0];
            ls.value[1] = ls.map[idx][1];
            ls.value[2] = ls.map[idx][2];
            ls.value[3] = 255; // ls.map[idx][3]
        }

        let color = i32::from_ne_bytes(ls.value);
        trap::R_SetLightStyle(ctx.engine, i as c_int, color);
    }
}

/// Raven `CG_SetLightstyle` — compiles configstring `i`'s lightstyle string
/// into `cl_lightstyle[i/3]`'s per-frame animation map.
///
/// Every char maps `'a'..'z'` onto a 0-255 brightness byte; `i%3` picks which
/// color channel this configstring feeds (three consecutive configstrings
/// drive R/G/B of one compiled style).
///
/// Source: `oracle/codemp/cgame/cg_light.c:68-85`
pub fn CG_SetLightstyle(ctx: &mut CgContext, i: c_int) {
    let s = CG_ConfigString(ctx, i + CS_LIGHT_STYLES);
    // raw wire bytes, not the decoded String's UTF-8 length - matches Raven's strlen
    let bytes = string_to_latin1(&s);
    let j = bytes.len();
    if j >= MAX_QPATH {
        let msg = format!("svc_lightstyle length={j}");
        Com_Error(ctx, errorParm_t::ERR_DROP as c_int, &msg);
        return;
    }

    let idx = (i / 3) as usize;
    let channel = (i % 3) as usize;
    ctx.world.light.cl_lightstyle[idx].length = j as i32;
    for k in 0..j {
        // Raven's `255.0` is a double literal, so the f32 quotient promotes to
        // double before the multiply and byte narrowing.
        ctx.world.light.cl_lightstyle[idx].map[k][channel] =
            (((bytes[k] as i32 - 'a' as i32) as f32 / ('z' as i32 - 'a' as i32) as f32) as f64
                * 255.0) as i32 as u8;
    }
}
