//! MP `botlib.h` bot input action flags (`bot_input_s::actionflags`).
//!
//! Plain `#define`s (not an enum), so §C8 makes them `const`s directly.
//! `ACTION_AFFIRMATIVE`/`ACTION_NEGATIVE`/`ACTION_GETFLAG`/`ACTION_GUARDBASE`/
//! `ACTION_PATROL`/`ACTION_FOLLOWME` are commented out (`/* ... */`) in the
//! Raven source and are not ported — they are dead, never-compiled code.
//!
//! Source: `oracle/codemp/game/botlib.h:65-82`

use core::ffi::c_int;

/// Raven `ACTION_ATTACK`.
///
/// Source: `oracle/codemp/game/botlib.h:66`
pub const ACTION_ATTACK: c_int = 0x0000001;

/// Raven `ACTION_USE`.
///
/// Source: `oracle/codemp/game/botlib.h:67`
pub const ACTION_USE: c_int = 0x0000002;

/// Raven `ACTION_RESPAWN`.
///
/// Source: `oracle/codemp/game/botlib.h:68`
pub const ACTION_RESPAWN: c_int = 0x0000008;

/// Raven `ACTION_JUMP`.
///
/// Source: `oracle/codemp/game/botlib.h:69`
pub const ACTION_JUMP: c_int = 0x0000010;

/// Raven `ACTION_MOVEUP`.
///
/// Source: `oracle/codemp/game/botlib.h:70`
pub const ACTION_MOVEUP: c_int = 0x0000020;

/// Raven `ACTION_CROUCH`.
///
/// Source: `oracle/codemp/game/botlib.h:71`
pub const ACTION_CROUCH: c_int = 0x0000080;

/// Raven `ACTION_MOVEDOWN`.
///
/// Source: `oracle/codemp/game/botlib.h:72`
pub const ACTION_MOVEDOWN: c_int = 0x0000100;

/// Raven `ACTION_MOVEFORWARD`.
///
/// Source: `oracle/codemp/game/botlib.h:73`
pub const ACTION_MOVEFORWARD: c_int = 0x0000200;

/// Raven `ACTION_MOVEBACK`.
///
/// Source: `oracle/codemp/game/botlib.h:74`
pub const ACTION_MOVEBACK: c_int = 0x0000800;

/// Raven `ACTION_MOVELEFT`.
///
/// Source: `oracle/codemp/game/botlib.h:75`
pub const ACTION_MOVELEFT: c_int = 0x0001000;

/// Raven `ACTION_MOVERIGHT`.
///
/// Source: `oracle/codemp/game/botlib.h:76`
pub const ACTION_MOVERIGHT: c_int = 0x0002000;

/// Raven `ACTION_DELAYEDJUMP`.
///
/// Source: `oracle/codemp/game/botlib.h:77`
pub const ACTION_DELAYEDJUMP: c_int = 0x0008000;

/// Raven `ACTION_TALK`.
///
/// Source: `oracle/codemp/game/botlib.h:78`
pub const ACTION_TALK: c_int = 0x0010000;

/// Raven `ACTION_GESTURE`.
///
/// Source: `oracle/codemp/game/botlib.h:79`
pub const ACTION_GESTURE: c_int = 0x0020000;

/// Raven `ACTION_WALK`.
///
/// Source: `oracle/codemp/game/botlib.h:80`
pub const ACTION_WALK: c_int = 0x0080000;

/// Raven `ACTION_FORCEPOWER`.
///
/// Source: `oracle/codemp/game/botlib.h:81`
pub const ACTION_FORCEPOWER: c_int = 0x0100000;

/// Raven `ACTION_ALT_ATTACK`.
///
/// Source: `oracle/codemp/game/botlib.h:82`
pub const ACTION_ALT_ATTACK: c_int = 0x0200000;
