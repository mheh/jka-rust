use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_SAVE_GOAL_FUZZY_LOGIC` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiSaveGoalFuzzyLogicArgs {
    pub goalstate: c_int,
    pub filename: CString,
}

impl BotlibAiSaveGoalFuzzyLogicArgs {
    pub fn new(goalstate: c_int, filename: CString) -> Self {
        Self {
            goalstate,
            filename,
        }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }

    pub fn filename(&self) -> &CString {
        &self.filename
    }
}

/// `BOTLIB_AI_SAVE_GOAL_FUZZY_LOGIC` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:460`
pub struct BotlibAiSaveGoalFuzzyLogic;

impl OutboundSysCall for BotlibAiSaveGoalFuzzyLogic {
    type Import = GameImport;
    type Args = BotlibAiSaveGoalFuzzyLogicArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_SAVE_GOAL_FUZZY_LOGIC;
}

impl EncodeSysCall for BotlibAiSaveGoalFuzzyLogic {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize, ptr_to_word(a.filename.as_ptr())])
    }
}

impl DecodeSysCallReturn for BotlibAiSaveGoalFuzzyLogic {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
