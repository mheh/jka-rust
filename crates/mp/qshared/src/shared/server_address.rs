use core::ffi::c_int;

/// Raven `#define AS_LOCAL 0` — the `ui_netSource`/server-browser address
/// source selector family.
/// Source: `oracle/codemp/game/q_shared.h:3025`
pub const AS_LOCAL: c_int = 0;
/// Raven `#define AS_GLOBAL 1`.
/// Source: `oracle/codemp/game/q_shared.h:3026`
pub const AS_GLOBAL: c_int = 1;
/// Raven `#define AS_FAVORITES 2`.
/// Source: `oracle/codemp/game/q_shared.h:3027`
pub const AS_FAVORITES: c_int = 2;
/// Raven `#define AS_MPLAYER 3` — Obsolete.
/// Source: `oracle/codemp/game/q_shared.h:3029`
pub const AS_MPLAYER: c_int = 3;
