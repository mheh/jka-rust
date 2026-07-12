#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::qcommon::bot_goal_t;

use crate::be_ai_weight::weightconfig_s::weightconfig_t;

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
/// Type definition source: `oracle/codemp/botlib/be_ai_goal.cpp:148-161`
#[repr(C)]
pub struct bot_goalstate_t {
    /// weight config
    pub itemweightconfig: *mut weightconfig_t,
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

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_goalstate_t>() == 2528);
    assert!(core::mem::offset_of!(bot_goalstate_t, itemweightconfig) == 0);
    assert!(core::mem::offset_of!(bot_goalstate_t, itemweightindex) == 8);
    assert!(core::mem::offset_of!(bot_goalstate_t, client) == 16);
    assert!(core::mem::offset_of!(bot_goalstate_t, lastreachabilityarea) == 20);
    assert!(core::mem::offset_of!(bot_goalstate_t, goalstack) == 24);
    assert!(core::mem::offset_of!(bot_goalstate_t, goalstacktop) == 472);
    assert!(core::mem::offset_of!(bot_goalstate_t, avoidgoals) == 476);
    assert!(core::mem::offset_of!(bot_goalstate_t, avoidgoaltimes) == 1500);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_goalstate_t>() == 2516);
    assert!(core::mem::offset_of!(bot_goalstate_t, itemweightconfig) == 0);
    assert!(core::mem::offset_of!(bot_goalstate_t, itemweightindex) == 4);
    assert!(core::mem::offset_of!(bot_goalstate_t, client) == 8);
    assert!(core::mem::offset_of!(bot_goalstate_t, lastreachabilityarea) == 12);
    assert!(core::mem::offset_of!(bot_goalstate_t, goalstack) == 16);
    assert!(core::mem::offset_of!(bot_goalstate_t, goalstacktop) == 464);
    assert!(core::mem::offset_of!(bot_goalstate_t, avoidgoals) == 468);
    assert!(core::mem::offset_of!(bot_goalstate_t, avoidgoaltimes) == 1492);
};
