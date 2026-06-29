use core::ffi::{c_int, c_void};
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_GET_MAP_LOCATION_GOAL` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotGetMapLocationGoal(char *name, void /*bot_goal_s*/ *goal)`
/// Syscall: `syscall(BOTLIB_AI_GET_MAP_LOCATION_GOAL, name, goal)`
#[derive(Debug)]
pub struct BotlibAiGetMapLocationGoalArgs {
    /// Location name string to look up.
    name: CString,
    /// Out-param: engine fills the opaque `bot_goal_s` struct through this pointer.
    goal: *mut c_void,
}

impl BotlibAiGetMapLocationGoalArgs {
    pub fn new(name: CString, goal: *mut c_void) -> Self {
        Self { name, goal }
    }

    pub fn name(&self) -> &CString {
        &self.name
    }

    pub fn goal(&self) -> *mut c_void {
        self.goal
    }
}

/// `BOTLIB_AI_GET_MAP_LOCATION_GOAL` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:486`
pub struct BotlibAiGetMapLocationGoal;

impl OutboundSysCall for BotlibAiGetMapLocationGoal {
    type Import = GameImport;
    type Args = BotlibAiGetMapLocationGoalArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_GET_MAP_LOCATION_GOAL;
}

impl EncodeSysCall for BotlibAiGetMapLocationGoal {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.name.as_ptr()), ptr_to_word(a.goal)])
    }
}

impl DecodeSysCallReturn for BotlibAiGetMapLocationGoal {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
