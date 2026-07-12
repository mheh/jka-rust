use core::ffi::c_int;

/// Raven `DEFAULT_MINS_2`.
///
/// Source: `oracle/codemp/game/bg_public.h:41`
pub const DEFAULT_MINS_2: c_int = -24;

/// Raven `DEFAULT_MAXS_2`.
///
/// Source: `oracle/codemp/game/bg_public.h:42`
pub const DEFAULT_MAXS_2: c_int = 40;

/// Raven `DEFAULT_VIEWHEIGHT` — `DEFAULT_MAXS_2 + STANDARD_VIEWHEIGHT_OFFSET`
/// (`40 + -4` = 36; the header's trailing `//26` comment is stale relative to
/// the macro's actual expansion).
///
/// Source: `oracle/codemp/game/bg_public.h:47`
pub const DEFAULT_VIEWHEIGHT: c_int = 36;

/// Raven `CROUCH_VIEWHEIGHT` — `CROUCH_MAXS_2 + STANDARD_VIEWHEIGHT_OFFSET`
/// (`16 + -4`).
///
/// Source: `oracle/codemp/game/bg_public.h:48`
pub const CROUCH_VIEWHEIGHT: c_int = 12;

/// Raven `DEAD_VIEWHEIGHT`.
///
/// Source: `oracle/codemp/game/bg_public.h:49`
pub const DEAD_VIEWHEIGHT: c_int = -16;
