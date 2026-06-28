use core::ffi::{c_int, c_void, c_char};
use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_GET_LEVEL_ITEM_GOAL` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotGetLevelItemGoal(int index, char *classname, bot_goal_t *goal)`
/// syscall: `BOTLIB_AI_GET_LEVEL_ITEM_GOAL, index, classname, goal`
#[derive(Debug)]
pub struct BotlibAiGetLevelItemGoalArgs {
    index: c_int,
    classname: *const c_char,
    goal: *mut c_void,
}

impl BotlibAiGetLevelItemGoalArgs {
    pub fn new(index: c_int, classname: *const c_char, goal: *mut c_void) -> Self {
        Self { index, classname, goal }
    }

    pub fn index(&self) -> c_int { self.index }
    pub fn classname(&self) -> *const c_char { self.classname }
    pub fn goal(&self) -> *mut c_void { self.goal }
}

pub struct BotlibAiGetLevelItemGoal;

impl OutboundSysCall for BotlibAiGetLevelItemGoal {
    type Args = BotlibAiGetLevelItemGoalArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_GET_LEVEL_ITEM_GOAL;
}

impl EncodeSysCall for BotlibAiGetLevelItemGoal {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.index as isize,
            ptr_to_word(a.classname),
            ptr_to_word(a.goal),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiGetLevelItemGoal {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
