use core::ffi::c_int;

/// Raven `#define CIN_system 1` — `trap_CIN_PlayCinematic` playback flags.
/// Source: `oracle/codemp/game/q_shared.h:515`
pub const CIN_SYSTEM: c_int = 1;
/// Raven `#define CIN_loop 2`.
/// Source: `oracle/codemp/game/q_shared.h:516`
pub const CIN_LOOP: c_int = 2;
/// Raven `#define CIN_hold 4`.
/// Source: `oracle/codemp/game/q_shared.h:517`
pub const CIN_HOLD: c_int = 4;
/// Raven `#define CIN_silent 8`.
/// Source: `oracle/codemp/game/q_shared.h:518`
pub const CIN_SILENT: c_int = 8;
/// Raven `#define CIN_shader 16`.
/// Source: `oracle/codemp/game/q_shared.h:519`
pub const CIN_SHADER: c_int = 16;
