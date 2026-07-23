#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::qcommon::bot_goal_t;

use crate::be_ai_weight::weightconfig_s::WeightConfigHandle;

/// `MAX_GOALSTACK`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp` (goal stack depth).
pub const MAX_GOALSTACK: usize = 8;

/// `MAX_AVOIDGOALS`.
///
/// Source: `oracle/codemp/botlib/be_ai_goal.cpp` (avoid-goal ring size).
pub const MAX_AVOIDGOALS: usize = 256;

/// Raven `bot_goalstate_t` — the goal state of a single bot.
///
/// `itemweightconfig` became a `WeightConfigHandle` into the `BotLib`
/// weight-config arena (porting-rules §F17); the struct is botlib-internal
/// (never crosses the ABI seam), so `#[repr(C)]` and its layout asserts are
/// dropped. It stays zero-valid (`None`/null/0) for `GetClearedMemory`.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_goal.cpp:148-161`
pub struct bot_goalstate_t {
    /// weight config
    pub itemweightconfig: Option<WeightConfigHandle>,
    /// index from item to weight
    pub itemweightindex: *mut i32,
    /// client using this goal state
    pub client: i32,
    /// last area with reachabilities the bot was in
    pub lastreachabilityarea: i32,
    /// goal stack
    pub goalstack: [bot_goal_t; MAX_GOALSTACK],
    /// the top of the goal stack
    pub goalstacktop: i32,
    /// goals to avoid
    pub avoidgoals: [i32; MAX_AVOIDGOALS],
    /// times to avoid the goals
    pub avoidgoaltimes: [f32; MAX_AVOIDGOALS],
}

pub type bot_goalstate_s = bot_goalstate_t;
