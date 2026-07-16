//! MP `bg_public.h` hyperspace transition constants.
//!
//! Source: `oracle/codemp/game/bg_public.h:1679-1680`

use core::ffi::c_int;

/// Raven `HYPERSPACE_TIME` — total duration of a hyperspace transition (msec).
/// Source: `oracle/codemp/game/bg_public.h:1679`
pub const HYPERSPACE_TIME: c_int = 4000;

/// Raven `HYPERSPACE_TELEPORT_FRAC` — fraction of `HYPERSPACE_TIME` at which the
/// teleport actually happens.
/// Source: `oracle/codemp/game/bg_public.h:1680`
pub const HYPERSPACE_TELEPORT_FRAC: f32 = 0.75;
