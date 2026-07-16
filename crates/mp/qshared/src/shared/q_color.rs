//! MP `q_shared.h` color-string helpers.
//!
//! Source: `oracle/codemp/game/q_shared.h:1145-1147`

#![allow(non_upper_case_globals, non_snake_case)]

use core::ffi::CStr;

/// Raven `Q_COLOR_ESCAPE` — the `^` sigil that opens a color code in a string.
///
/// Source: `oracle/codemp/game/q_shared.h:1145`
pub const Q_COLOR_ESCAPE: u8 = b'^';

// Raven color escape `#define`s (porting-rules §C8: `#define` -> `const`).
// Relocated here from `mp_game`'s `g_team` (shared `q_shared.h` header) so the
// bg crate can reach them; `g_team` re-exports them so game importers are
// unchanged.
// Source: oracle/codemp/game/q_shared.h:1145-1167
pub const S_COLOR_RED: &CStr = c"^1";
pub const S_COLOR_GREEN: &CStr = c"^2";
pub const S_COLOR_YELLOW: &CStr = c"^3";
pub const S_COLOR_BLUE: &CStr = c"^4";
pub const S_COLOR_WHITE: &CStr = c"^7";

/// Raven `Q_IsColorString(p)` — true if `p` points at a `^N` color code
/// (`N` in `'0'..='7'`).
///
/// Raven comment: "you MUST have the last bit on here about colour strings
/// being less than 7 or taiwanese strings register as colour!!!!"
/// Source: `oracle/codemp/game/q_shared.h:1147`
pub fn Q_IsColorString(p: &[u8]) -> bool {
    p.first() == Some(&Q_COLOR_ESCAPE)
        && matches!(p.get(1), Some(&c) if c != Q_COLOR_ESCAPE && (b'0'..=b'7').contains(&c))
}
