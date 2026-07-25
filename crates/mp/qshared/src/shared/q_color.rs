//! MP `q_shared.h` color-string helpers.
//!
//! Source: `oracle/codemp/game/q_shared.h:1145-1147`

#![allow(non_upper_case_globals, non_snake_case)]

use core::ffi::CStr;

use crate::shared::vec4_t;

/// Raven `colorBlack`.
/// Source: `oracle/codemp/game/q_math.c:11`
pub const colorBlack: vec4_t = [0.0, 0.0, 0.0, 1.0];
/// Raven `colorRed`.
/// Source: `oracle/codemp/game/q_math.c:12`
pub const colorRed: vec4_t = [1.0, 0.0, 0.0, 1.0];
/// Raven `colorGreen`.
/// Source: `oracle/codemp/game/q_math.c:13`
pub const colorGreen: vec4_t = [0.0, 1.0, 0.0, 1.0];
/// Raven `colorBlue`.
/// Source: `oracle/codemp/game/q_math.c:14`
pub const colorBlue: vec4_t = [0.0, 0.0, 1.0, 1.0];
/// Raven `colorYellow`.
/// Source: `oracle/codemp/game/q_math.c:15`
pub const colorYellow: vec4_t = [1.0, 1.0, 0.0, 1.0];
/// Raven `colorMagenta`.
/// Source: `oracle/codemp/game/q_math.c:16`
pub const colorMagenta: vec4_t = [1.0, 0.0, 1.0, 1.0];
/// Raven `colorCyan`.
/// Source: `oracle/codemp/game/q_math.c:17`
pub const colorCyan: vec4_t = [0.0, 1.0, 1.0, 1.0];
/// Raven `colorWhite`.
/// Source: `oracle/codemp/game/q_math.c:18`
pub const colorWhite: vec4_t = [1.0, 1.0, 1.0, 1.0];
/// Raven `colorLtGrey`.
/// Source: `oracle/codemp/game/q_math.c:19`
pub const colorLtGrey: vec4_t = [0.75, 0.75, 0.75, 1.0];
/// Raven `colorMdGrey`.
/// Source: `oracle/codemp/game/q_math.c:20`
pub const colorMdGrey: vec4_t = [0.5, 0.5, 0.5, 1.0];
/// Raven `colorDkGrey`.
/// Source: `oracle/codemp/game/q_math.c:21`
pub const colorDkGrey: vec4_t = [0.25, 0.25, 0.25, 1.0];

/// Raven `g_color_table[8]`.
/// Source: `oracle/codemp/game/q_math.c:26-35`
pub const g_color_table: [vec4_t; 8] = [
    [0.0, 0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0, 1.0],
    [0.0, 1.0, 0.0, 1.0],
    [1.0, 1.0, 0.0, 1.0],
    [0.0, 0.0, 1.0, 1.0],
    [0.0, 1.0, 1.0, 1.0],
    [1.0, 0.0, 1.0, 1.0],
    [1.0, 1.0, 1.0, 1.0],
];

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
