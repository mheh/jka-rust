//! MP `q_shared.h` color-string helpers.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:1145-1147`

#![allow(non_upper_case_globals, non_snake_case)]

/// Raven `Q_COLOR_ESCAPE` — the `^` sigil that opens a color code in a string.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:1145`
pub const Q_COLOR_ESCAPE: u8 = b'^';

/// Raven `Q_IsColorString(p)` — true if `p` points at a `^N` color code
/// (`N` in `'0'..='7'`).
///
/// Raven comment: "you MUST have the last bit on here about colour strings
/// being less than 7 or taiwanese strings register as colour!!!!"
/// Source: `oracle/oracle/codemp/game/q_shared.h:1147`
pub fn Q_IsColorString(p: &[u8]) -> bool {
    p.first() == Some(&Q_COLOR_ESCAPE)
        && matches!(p.get(1), Some(&c) if c != Q_COLOR_ESCAPE && (b'0'..=b'7').contains(&c))
}
