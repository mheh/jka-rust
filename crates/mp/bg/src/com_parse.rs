//! bg-tier twins of the `q_shared.c` `COM_ParseString`/`COM_ParseInt`/
//! `COM_ParseFloat` helpers that `bg_saberLoad.c` consumes.
//!
//! The canonical (game-tier) copies live in `mp_game::q_shared` and route their
//! "unexpected EOF" diagnostic through the game-tier `Com_Printf`. The bg copies
//! here are the same parse logic, but the EOF print is routed through the
//! `BgTraps` seam (`traps.com_printf`), and the shared parse state / numeric
//! conversions come from the lower tiers (`mp_qshared`'s `COM_ParseExt`,
//! `native_string`'s `atoi`/`atof`). Game copies untouched.
//!
//! Phase-5b: byte-cursor shape — the `char**` cursor becomes
//! `&mut Option<&[u8]>`, the `char**`/numeric out-params become `&mut String` /
//! `&mut c_int` / `&mut f32`.
//!
//! Source: `oracle/codemp/game/q_shared.c:588-638`
#![allow(non_snake_case)]

use mp_qshared::shared::com_parse::{COM_ParseExt, QSharedScratch};
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use native_string::atof::atof_bytes;
use native_string::atoi::atoi_bytes;

use crate::bg_channel::bg_traps::BgTraps;
use core::ffi::c_int;

/// Raven `COM_ParseString` — bg twin routing the EOF print via `traps`.
///
/// Raven's guard is `if ( s[0] == 0 )` where `s` is `const char **`, so `s[0]`
/// is the (always non-NULL) `com_token` pointer, not the first token byte — the
/// EOF branch is dead and `COM_ParseString` never returns `qtrue`. Preserved.
/// Source: `oracle/codemp/game/q_shared.c:588-598`
pub fn COM_ParseString(
    qs: &mut QSharedScratch,
    data: &mut Option<&[u8]>,
    s: &mut String,
    _traps: &dyn BgTraps,
) -> qboolean {
    let (token, rest) = COM_ParseExt(qs, *data, false);
    *data = rest;
    *s = token;
    qfalse
}

/// Raven `COM_ParseInt` — bg twin routing the EOF print via `traps`.
///
/// Raven's guard is `if ( token[0] == 0 )` (empty token), so this DOES fire on
/// EOF / a line break.
/// Source: `oracle/codemp/game/q_shared.c:605-618`
pub fn COM_ParseInt(
    qs: &mut QSharedScratch,
    data: &mut Option<&[u8]>,
    i: &mut c_int,
    traps: &dyn BgTraps,
) -> qboolean {
    let (token, rest) = COM_ParseExt(qs, *data, false);
    *data = rest;
    if token.is_empty() {
        traps.com_printf("unexpected EOF\n");
        return qtrue;
    }
    *i = atoi_bytes(token.as_bytes());
    qfalse
}

/// Raven `COM_ParseFloat` — bg twin routing the EOF print via `traps`.
/// Source: `oracle/codemp/game/q_shared.c:625-638`
pub fn COM_ParseFloat(
    qs: &mut QSharedScratch,
    data: &mut Option<&[u8]>,
    f: &mut f32,
    traps: &dyn BgTraps,
) -> qboolean {
    let (token, rest) = COM_ParseExt(qs, *data, false);
    *data = rest;
    if token.is_empty() {
        traps.com_printf("unexpected EOF\n");
        return qtrue;
    }
    *f = atof_bytes(token.as_bytes()) as f32;
    qfalse
}
