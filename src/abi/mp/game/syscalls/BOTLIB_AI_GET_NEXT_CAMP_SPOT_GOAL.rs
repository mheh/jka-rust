use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_GET_NEXT_CAMP_SPOT_GOAL` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiGetNextCampSpotGoalArgs {
    num: c_int,
    goal: *mut c_void,
}

impl BotlibAiGetNextCampSpotGoalArgs {
    pub fn new(num: c_int, goal: *mut c_void) -> Self {
        Self { num, goal }
    }

    pub fn num(&self) -> c_int {
        self.num
    }

    pub fn goal(&self) -> *mut c_void {
        self.goal
    }
}

/// `BOTLIB_AI_GET_NEXT_CAMP_SPOT_GOAL` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:485`
pub struct BotlibAiGetNextCampSpotGoal;

impl OutboundSysCall for BotlibAiGetNextCampSpotGoal {
    type Import = GameImport;
    type Args = BotlibAiGetNextCampSpotGoalArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_GET_NEXT_CAMP_SPOT_GOAL;
}

impl EncodeSysCall for BotlibAiGetNextCampSpotGoal {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.num as isize, ptr_to_word(a.goal)])
    }
}

impl DecodeSysCallReturn for BotlibAiGetNextCampSpotGoal {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
