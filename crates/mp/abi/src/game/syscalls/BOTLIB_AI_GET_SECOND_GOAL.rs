use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_GET_SECOND_GOAL` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotGetSecondGoal(int goalstate, void *goal)`
#[derive(Debug)]
pub struct BotlibAiGetSecondGoalArgs {
    /// Handle to the bot goal state.
    goalstate: c_int,
    /// Out-param: engine writes the second goal through this pointer.
    goal: *mut core::ffi::c_void,
}

impl BotlibAiGetSecondGoalArgs {
    pub fn new(goalstate: c_int, goal: *mut core::ffi::c_void) -> Self {
        Self { goalstate, goal }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }

    pub fn goal(&self) -> *mut core::ffi::c_void {
        self.goal
    }
}

/// `BOTLIB_AI_GET_SECOND_GOAL` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:449`
pub struct BotlibAiGetSecondGoal;

impl OutboundSysCall for BotlibAiGetSecondGoal {
    type Import = MpGameImport;
    type Args = BotlibAiGetSecondGoalArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_GET_SECOND_GOAL;
}

impl EncodeSysCall for BotlibAiGetSecondGoal {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize, ptr_to_word(a.goal)])
    }
}

impl DecodeSysCallReturn for BotlibAiGetSecondGoal {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
