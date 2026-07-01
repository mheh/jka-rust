use core::ffi::c_int;

use super::super::MpGameImport;
use mp_qshared::common::mp::qcommon::bot_goal_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `BOTLIB_AI_GET_TOP_GOAL`.
///
/// `goal` is an OUT-PARAM: the engine fills the caller-provided [`bot_goal_t`]
/// in place, so it stays a raw pointer here rather than becoming a return value.
#[derive(Debug)]
pub struct BotlibAiGetTopGoalArgs {
    goalstate: c_int,
    goal: *mut bot_goal_t,
}

impl BotlibAiGetTopGoalArgs {
    pub fn new(goalstate: c_int, goal: *mut bot_goal_t) -> Self {
        Self { goalstate, goal }
    }

    pub const fn goalstate(&self) -> c_int {
        self.goalstate
    }

    pub const fn goal(&self) -> *mut bot_goal_t {
        self.goal
    }
}

/// `BOTLIB_AI_GET_TOP_GOAL` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:448`
pub struct BotlibAiGetTopGoal;

impl OutboundSysCall for BotlibAiGetTopGoal {
    type Import = MpGameImport;
    type Args = BotlibAiGetTopGoalArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_GET_TOP_GOAL;
}

impl EncodeSysCall for BotlibAiGetTopGoal {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.goalstate() as isize, ptr_to_word(args.goal())])
    }
}

impl DecodeSysCallReturn for BotlibAiGetTopGoal {
    // `trap_BotGetTopGoal` returns `int`.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
