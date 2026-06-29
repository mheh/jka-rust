use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_INTERBREED_GOAL_FUZZY_LOGIC` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotInterbreedGoalFuzzyLogic(int parent1, int parent2, int child)`
#[derive(Debug)]
pub struct BotlibAiInterbreedGoalFuzzyLogicArgs {
    parent1: c_int,
    parent2: c_int,
    child: c_int,
}

impl BotlibAiInterbreedGoalFuzzyLogicArgs {
    pub fn new(parent1: c_int, parent2: c_int, child: c_int) -> Self {
        Self {
            parent1,
            parent2,
            child,
        }
    }

    pub fn parent1(&self) -> c_int {
        self.parent1
    }
    pub fn parent2(&self) -> c_int {
        self.parent2
    }
    pub fn child(&self) -> c_int {
        self.child
    }
}

/// `BOTLIB_AI_INTERBREED_GOAL_FUZZY_LOGIC` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:483`
pub struct BotlibAiInterbreedGoalFuzzyLogic;

impl OutboundSysCall for BotlibAiInterbreedGoalFuzzyLogic {
    type Import = GameImport;
    type Args = BotlibAiInterbreedGoalFuzzyLogicArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_INTERBREED_GOAL_FUZZY_LOGIC;
}

impl EncodeSysCall for BotlibAiInterbreedGoalFuzzyLogic {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.parent1 as isize, a.parent2 as isize, a.child as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiInterbreedGoalFuzzyLogic {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
