//! SP `saber_colors_t`.
//!
//! Type definition source: `oracle/oracle/code/game/q_shared.h:474-483`

#![allow(non_camel_case_types)]

/// Raven SP `saber_colors_t`.
///
/// Unlike MP (which uses `typedef int` + an anonymous enum), SP declares the
/// enum itself as the type, and has **no** `NUM_SABER_COLORS` terminator.
/// Type definition source: `oracle/oracle/code/game/q_shared.h:474-483`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum saber_colors_t {
    SABER_RED = 0,
    SABER_ORANGE,
    SABER_YELLOW,
    SABER_GREEN,
    SABER_BLUE,
    SABER_PURPLE,
}
