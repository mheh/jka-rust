// PORT-COMPLETE: NPC_misc.c 2/2

//! FAITHFUL port of `oracle/codemp/game/NPC_misc.c`.
//!
//! Debug logging functions for NPC AI. Both functions thread GameContext
//! as the first parameter and access level.time from the game world.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_main::Com_Printf;
use crate::prelude::*;

// Raven DEBUG_LEVEL_* constants from b_local.h. The former values here
// (DETAIL 1 / WARNING 5 / ERROR 7) were guessed and inverted the severity
// ordering the `cv->value < debugLevel` gate depends on — a live bug — so they
// are corrected to the oracle values.
// Source: oracle/codemp/game/b_local.h:22-25
const DEBUG_LEVEL_DETAIL: c_int = 4;
const DEBUG_LEVEL_INFO: c_int = 3;
const DEBUG_LEVEL_WARNING: c_int = 2;
const DEBUG_LEVEL_ERROR: c_int = 1;

/// Raven `Debug_Printf`.
///
/// Debug logging function that formats a message if the debug level is enabled,
/// then prints it with timestamp prefix. The cvar is checked first to gate output.
///
/// Source: `oracle/codemp/game/NPC_misc.c:10-35`
pub fn Debug_Printf(
    ctx: GameContext<'_>,
    cv: *mut vmCvar_t,
    debugLevel: c_int,
    fmt: *mut c_char,
    // variadic `...` — C varargs, seam decision pending
) {
    // PORT-NOTE(varargs): C varargs cannot be captured in safe Rust functions.
    // In actual Rust call sites (NPC_combat.rs), this is called with a
    // pre-formatted message string; the variadic arguments are not used.

    unsafe {
        // Check if cvar value is less than debug level; if so, don't print
        if (*cv).value < debugLevel as f32 {
            return;
        }

        let color: &'static str = match debugLevel {
            DEBUG_LEVEL_DETAIL => "^7",  // S_COLOR_WHITE
            DEBUG_LEVEL_INFO => "^2",    // S_COLOR_GREEN
            DEBUG_LEVEL_WARNING => "^3", // S_COLOR_YELLOW
            DEBUG_LEVEL_ERROR => "^1",   // S_COLOR_RED
            _ => "^1",                   // Default to S_COLOR_RED
        };

        let time = (*ctx.world).level.time;
        let msg: String = cstr_to_str(fmt);

        let output = format!("{}{:5}:{}", color, time, msg);
        let output_cstr = cstr(&output);

        Com_Printf(output_cstr.as_ptr());
    }
}

/// Raven `Debug_NPCPrintf`.
///
/// Debug logging function similar to Debug_Printf, but adds NPC identification
/// (name and Q_COLOR_ESCAPE prefix) to the output. Format: `^c^t^i (npc) msg`
/// where c is the color escape, t is time, i is the NPC targetname.
///
/// Source: `oracle/codemp/game/NPC_misc.c:41-73`
pub fn Debug_NPCPrintf(
    ctx: GameContext<'_>,
    printNPC: EntityId,
    cv: *mut vmCvar_t,
    debugLevel: c_int,
    fmt: *mut c_char,
    // variadic `...` — C varargs, seam decision pending
) {
    // PORT-NOTE(varargs): C varargs cannot be captured in safe Rust functions.
    // In actual Rust call sites, this is called with pre-formatted strings.

    // STAGE-1: EntityId param, raw body re-derived verbatim (Stage-2 debt).
    let printNPC: *mut gentity_t = ctx.entity_mut(printNPC);
    unsafe {
        // Check if cvar value is less than debug level; if so, don't print
        if (*cv).value < debugLevel as f32 {
            return;
        }

        // Map debug level to color code. Raven's COLOR_* are the ASCII digit
        // chars '7'/'2'/'3'/'1', emitted as literal bytes (not control codes).
        let color: char = match debugLevel {
            DEBUG_LEVEL_DETAIL => '7',  // COLOR_WHITE
            DEBUG_LEVEL_INFO => '2',    // COLOR_GREEN
            DEBUG_LEVEL_WARNING => '3', // COLOR_YELLOW
            DEBUG_LEVEL_ERROR => '1',   // COLOR_RED
            _ => '1',                   // Default to COLOR_RED
        };

        let time = (*ctx.world).level.time;
        let msg: String = cstr_to_str(fmt);
        let npc_targetname: String = cstr_to_str((*printNPC).targetname);

        // Format: Q_COLOR_ESCAPE ('^') + color char + time + NPC name + message
        let output = format!("^{}{:5} ({}) {}", color, time, npc_targetname, msg);
        let output_cstr = cstr(&output);

        Com_Printf(output_cstr.as_ptr());
    }
}
