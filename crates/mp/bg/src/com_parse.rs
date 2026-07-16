//! bg-tier twins of the `q_shared.c` `COM_ParseString`/`COM_ParseInt`/
//! `COM_ParseFloat` helpers that `bg_saberLoad.c` consumes.
//!
//! The canonical (game-tier) copies live in `mp_game::q_shared` and route their
//! "unexpected EOF" diagnostic through the game-tier `Com_Printf`. The bg copies
//! here are byte-for-byte the same parse logic, but the EOF print is routed
//! through the `BgTraps` seam (`traps.com_printf`), and the shared parse state /
//! numeric conversions come from the lower tiers (`mp_qshared`'s `COM_ParseExt`,
//! this crate's `cstr_util::atoi` and `bg_lib::atof`). Game copies untouched.
//!
//! Source: `oracle/codemp/game/q_shared.c:588-638`
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_qshared::shared::com_parse::{COM_ParseExt, QSharedScratch};
use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::bg_channel::bg_traps::BgTraps;
use crate::bg_lib::atof;
use crate::cstr_util::atoi;

/// Raven `COM_ParseString` — bg twin routing the EOF print via `traps`.
/// Source: `oracle/codemp/game/q_shared.c:588-598`
pub fn COM_ParseString(
    qs: &mut QSharedScratch,
    data: *mut *const c_char,
    s: *mut *const c_char,
    traps: &dyn BgTraps,
) -> qboolean {
    unsafe {
        let token = COM_ParseExt(qs, data, qfalse);
        *s = token as *const c_char;
        // Raven's guard is literally `if ( s[0] == 0 )` — `s` is `const char **`,
        // so `s[0]` is the token pointer itself, not `*token`. That's always
        // non-zero here (COM_ParseExt never returns NULL), so the oracle's check
        // is dead in practice; preserved faithfully as a null-pointer check.
        if (*s).is_null() {
            traps.com_printf(c"unexpected EOF\n".as_ptr());
            return qtrue;
        }
        qfalse
    }
}

/// Raven `COM_ParseInt` — bg twin routing the EOF print via `traps`.
/// Source: `oracle/codemp/game/q_shared.c:605-618`
pub fn COM_ParseInt(
    qs: &mut QSharedScratch,
    data: *mut *const c_char,
    i: *mut c_int,
    traps: &dyn BgTraps,
) -> qboolean {
    unsafe {
        let token = COM_ParseExt(qs, data, qfalse);
        if *token == 0 {
            traps.com_printf(c"unexpected EOF\n".as_ptr());
            return qtrue;
        }
        *i = atoi(token as *const c_char);
        qfalse
    }
}

/// Raven `COM_ParseFloat` — bg twin routing the EOF print via `traps`.
/// Source: `oracle/codemp/game/q_shared.c:625-638`
pub fn COM_ParseFloat(
    qs: &mut QSharedScratch,
    data: *mut *const c_char,
    f: *mut f32,
    traps: &dyn BgTraps,
) -> qboolean {
    unsafe {
        let token = COM_ParseExt(qs, data, qfalse);
        if *token == 0 {
            traps.com_printf(c"unexpected EOF\n".as_ptr());
            return qtrue;
        }
        *f = atof(token as *const c_char) as f32;
        qfalse
    }
}
