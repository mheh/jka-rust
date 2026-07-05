//! MP `bg_public.h` Ghoul2 model parts definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:126-134`

#![allow(non_camel_case_types)]

/// Raven `g2ModelParts_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:126-134`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum g2ModelParts_t {
    G2_MODELPART_HEAD = 10,
    G2_MODELPART_WAIST = 11,
    G2_MODELPART_LARM = 12,
    G2_MODELPART_RARM = 13,
    G2_MODELPART_RHAND = 14,
    G2_MODELPART_LLEG = 15,
    G2_MODELPART_RLEG = 16,
}

use core::ffi::c_int;

/// Raven `G2_MODEL_PART` — sentinel weapon index meaning "use the Ghoul2 model
/// part", distinct from the `g2ModelParts_t` enum above.
///
/// Source: `oracle/oracle/codemp/game/bg_public.h:136`
pub const G2_MODEL_PART: c_int = 50;
