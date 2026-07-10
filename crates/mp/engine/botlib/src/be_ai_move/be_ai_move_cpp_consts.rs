#![allow(non_camel_case_types)]

//! `be_ai_move.cpp`-local movement-AI constants.
//!
//! Source: `oracle/codemp/botlib/be_ai_move.cpp:75-87`

/// Raven `AVOIDREACH` — unconditionally-defined feature guard: reachabilities
/// used within `AVOIDREACH_TIME` are avoided. Ported as `bool` since Raven
/// never gives it a value, only tests it with `#ifdef`; it is defined
/// unconditionally at this site, so the guarded branches always compile in.
///
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:74`
pub const AVOIDREACH: bool = true;

/// Raven `AVOIDREACH_TIME` — avoid links for 6 seconds after use.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:75`
pub const AVOIDREACH_TIME: i32 = 6;
/// Raven `AVOIDREACH_TRIES`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:76`
pub const AVOIDREACH_TRIES: i32 = 4;

/// Raven `PREDICTIONTIME_JUMP` — in seconds.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:78`
pub const PREDICTIONTIME_JUMP: f32 = 3.0;
/// Raven `PREDICTIONTIME_MOVE` — in seconds.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:79`
pub const PREDICTIONTIME_MOVE: f32 = 2.0;

/// Raven `WEAPONINDEX_ROCKET_LAUNCHER`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:81`
pub const WEAPONINDEX_ROCKET_LAUNCHER: i32 = 5;
/// Raven `WEAPONINDEX_BFG`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:82`
pub const WEAPONINDEX_BFG: i32 = 9;

/// Raven `MODELTYPE_FUNC_PLAT`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:84`
pub const MODELTYPE_FUNC_PLAT: i32 = 1;
/// Raven `MODELTYPE_FUNC_BOB`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:85`
pub const MODELTYPE_FUNC_BOB: i32 = 2;
/// Raven `MODELTYPE_FUNC_DOOR`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:86`
pub const MODELTYPE_FUNC_DOOR: i32 = 3;
/// Raven `MODELTYPE_FUNC_STATIC`.
/// Source: `oracle/codemp/botlib/be_ai_move.cpp:87`
pub const MODELTYPE_FUNC_STATIC: i32 = 4;
