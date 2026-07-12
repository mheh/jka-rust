#![allow(non_camel_case_types)]

//! `be_ai_goal.cpp`-local goal-AI constants.
//!
//! Source: `oracle/codemp/botlib/be_ai_goal.cpp:34-50`

/// Raven `UNDECIDEDFUZZY` — feature guard, only defined `#ifdef RANDOMIZE`.
/// Ported as `bool` since Raven never gives it a value, only tests it with
/// `#ifdef`. `RANDOMIZE` is defined unconditionally
/// (`oracle/codemp/botlib/be_interface.h:16`), so this is always on.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:34`
pub const UNDECIDEDFUZZY: bool = true;

/// Raven `DROPPEDWEIGHT` — unconditionally-defined feature guard. Ported as
/// `bool` for the same reason as [`UNDECIDEDFUZZY`].
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:36`
pub const DROPPEDWEIGHT: bool = true;

/// Raven `AVOID_MINIMUM_TIME` — minimum avoid goal time.
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:38`
pub const AVOID_MINIMUM_TIME: i32 = 10;

/// Raven `AVOID_DEFAULT_TIME` — default avoid goal time.
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:40`
pub const AVOID_DEFAULT_TIME: i32 = 30;

/// Raven `AVOID_DROPPED_TIME` — avoid dropped goal time.
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:42`
pub const AVOID_DROPPED_TIME: i32 = 10;

/// Raven `TRAVELTIME_SCALE`.
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:44`
pub const TRAVELTIME_SCALE: f32 = 0.01;

// Item flags — `IFL_*` (anonymous `#define` family).
/// Raven `IFL_NOTFREE` — not in free for all.
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:46`
pub const IFL_NOTFREE: i32 = 1;
/// Raven `IFL_NOTTEAM` — not in team play.
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:47`
pub const IFL_NOTTEAM: i32 = 2;
/// Raven `IFL_NOTSINGLE` — not in single player.
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:48`
pub const IFL_NOTSINGLE: i32 = 4;
/// Raven `IFL_NOTBOT` — bot should never go for this.
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:49`
pub const IFL_NOTBOT: i32 = 8;
/// Raven `IFL_ROAM` — bot roam goal.
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp:50`
pub const IFL_ROAM: i32 = 16;
