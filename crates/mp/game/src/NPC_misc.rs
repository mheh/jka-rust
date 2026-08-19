//! Debug logging functions for NPC AI.
//!
//! Both functions thread `GameContext` as the first parameter and read `level.time` from the game world.
#![allow(non_snake_case, unused, clippy::all)]

use crate::g_main::Com_Printf;
use crate::prelude::*;

// Raven DEBUG_LEVEL_* constants from b_local.h.
// The severity order must match the oracle values, because the `cv->value < debugLevel` gate depends on it.
// Source: oracle/codemp/game/b_local.h:22-25
const DEBUG_LEVEL_DETAIL: c_int = 4;
const DEBUG_LEVEL_INFO: c_int = 3;
const DEBUG_LEVEL_WARNING: c_int = 2;
const DEBUG_LEVEL_ERROR: c_int = 1;

/// Raven `Debug_Printf`.
///
/// This prints a message with a color and time prefix when the debug cvar allows the given level.
///
/// Source: `oracle/codemp/game/NPC_misc.c:10-35`
pub fn Debug_Printf(
    ctx: &mut GameContext,
    cv: *mut vmCvar_t,
    debugLevel: c_int,
    fmt: *mut c_char,
    // variadic `...`, C var args do not cross the Rust ABI boundary
) {
    // C var args cannot be captured in safe Rust.
    // Call sites (NPC_combat.rs) pre-format the message and pass it here.

    unsafe {
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

        let time = ctx.world.level.time;
        let msg: String = cstr_to_str(fmt);

        let output = format!("{}{:5}:{}", color, time, msg);

        Com_Printf(&output);
    }
}

/// Raven `Debug_NPCPrintf`.
///
/// This is like `Debug_Printf`, but it adds the NPC name and a `Q_COLOR_ESCAPE` prefix to the output.
/// Format: `^c^t^i (npc) msg`, where c is the color escape, t is time, and i is the NPC targetname.
///
/// Source: `oracle/codemp/game/NPC_misc.c:41-73`
pub fn Debug_NPCPrintf(
    ctx: &mut GameContext,
    printNPC: EntityId,
    cv: *mut vmCvar_t,
    debugLevel: c_int,
    fmt: *mut c_char,
    // variadic `...`, C var args do not cross the Rust ABI boundary
) {
    // C var args cannot be captured in safe Rust.
    // Call sites pre-format the message and pass it here.

    unsafe {
        if (*cv).value < debugLevel as f32 {
            return;
        }

        // Raven's COLOR_* values are the ASCII digit characters '7'/'2'/'3'/'1', emitted as literal bytes, not control codes.
        let color: char = match debugLevel {
            DEBUG_LEVEL_DETAIL => '7',  // COLOR_WHITE
            DEBUG_LEVEL_INFO => '2',    // COLOR_GREEN
            DEBUG_LEVEL_WARNING => '3', // COLOR_YELLOW
            DEBUG_LEVEL_ERROR => '1',   // COLOR_RED
            _ => '1',                   // Default to COLOR_RED
        };

        let time = ctx.world.level.time;
        let msg: String = cstr_to_str(fmt);
        let npc_targetname: String = ctx.entity(printNPC).targetname_str().unwrap_or_default();

        // Format: Q_COLOR_ESCAPE ('^') + color char + time + NPC name + message
        let output = format!("^{}{:5} ({}) {}", color, time, npc_targetname, msg);

        Com_Printf(&output);
    }
}
