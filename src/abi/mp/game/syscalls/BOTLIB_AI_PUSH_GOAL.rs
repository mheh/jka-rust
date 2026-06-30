use core::ffi::c_int;

use super::super::MpGameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::common::mp::qcommon::bot_goal_t;

/// Args for the `BOTLIB_AI_PUSH_GOAL` game→engine syscall.
///
/// Mirrors `trap_BotPushGoal(goalstate: i32, goal: *const bot_goal_t)`.
#[derive(Debug)]
pub struct BotlibAiPushGoalArgs {
    pub goalstate: c_int,
    pub goal: *const bot_goal_t,
}

impl BotlibAiPushGoalArgs {
    pub fn new(goalstate: c_int, goal: *const bot_goal_t) -> Self {
        Self { goalstate, goal }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }

    pub fn goal(&self) -> *const bot_goal_t {
        self.goal
    }
}

/// `BOTLIB_AI_PUSH_GOAL` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:442`
pub struct BotlibAiPushGoal;

impl OutboundSysCall for BotlibAiPushGoal {
    type Import = MpGameImport;
    type Args = BotlibAiPushGoalArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_PUSH_GOAL;
}

impl EncodeSysCall for BotlibAiPushGoal {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize, ptr_to_word(a.goal)])
    }
}

impl DecodeSysCallReturn for BotlibAiPushGoal {
    fn decode_return(_word: isize) -> Self::Output {}
}
