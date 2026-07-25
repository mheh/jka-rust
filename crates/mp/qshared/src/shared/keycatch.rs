use core::ffi::c_int;

/// Raven `#define KEYCATCH_CONSOLE 0x0001` — `trap_Key_SetCatcher` bits.
/// Source: `oracle/codemp/game/q_shared.h:1936`
pub const KEYCATCH_CONSOLE: c_int = 0x0001;
/// Raven `#define KEYCATCH_UI 0x0002`.
/// Source: `oracle/codemp/game/q_shared.h:1937`
pub const KEYCATCH_UI: c_int = 0x0002;
/// Raven `#define KEYCATCH_MESSAGE 0x0004`.
/// Source: `oracle/codemp/game/q_shared.h:1938`
pub const KEYCATCH_MESSAGE: c_int = 0x0004;
/// Raven `#define KEYCATCH_CGAME 0x0008`.
/// Source: `oracle/codemp/game/q_shared.h:1939`
pub const KEYCATCH_CGAME: c_int = 0x0008;
