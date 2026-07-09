//! `Eorientations` — cardinal-axis orientation selector.
//!
//! Identical values in SP and MP (MP writes `typedef enum Eorientations`, SP
//! `enum Eorientations` — same layout). Note the enumerator order is X, **Z, Y**.
//! Type definition source: `oracle/code/game/q_shared.h:2641-2650`
//! Type definition source: `oracle/codemp/game/q_shared.h:3086-3095`

#![allow(non_camel_case_types)]

/// Raven `Eorientations`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eorientations {
    ORIGIN = 0,
    POSITIVE_X,
    POSITIVE_Z,
    POSITIVE_Y,
    NEGATIVE_X,
    NEGATIVE_Z,
    NEGATIVE_Y,
}
