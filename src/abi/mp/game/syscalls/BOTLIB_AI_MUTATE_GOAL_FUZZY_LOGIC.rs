use crate::ffi::GameImport;

use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_MUTATE_GOAL_FUZZY_LOGIC` outbound game-to-engine syscall.
///
/// Mirrors `trap_BotMutateGoalFuzzyLogic(goalstate: i32, range: f32)`.
/// Note: the C wrapper passes `range` as a plain widened integer value
/// (not via `PASSFLOAT`), so we reproduce that verbatim.
#[derive(Debug)]
pub struct BotlibAiMutateGoalFuzzyLogicArgs {
    goalstate: i32,
    range: f32,
}

impl BotlibAiMutateGoalFuzzyLogicArgs {
    pub fn new(goalstate: i32, range: f32) -> Self {
        Self { goalstate, range }
    }

    pub fn goalstate(&self) -> i32 {
        self.goalstate
    }

    pub fn range(&self) -> f32 {
        self.range
    }
}

/// `BOTLIB_AI_MUTATE_GOAL_FUZZY_LOGIC` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:484`
pub struct BotlibAiMutateGoalFuzzyLogic;

impl OutboundSysCall for BotlibAiMutateGoalFuzzyLogic {
    type Import = GameImport;
    type Args = BotlibAiMutateGoalFuzzyLogicArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_MUTATE_GOAL_FUZZY_LOGIC;
}

impl EncodeSysCall for BotlibAiMutateGoalFuzzyLogic {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        // range is passed as a plain widened integer (not PASSFLOAT) per C wrapper.
        SysCallTransport::new([a.goalstate as isize, a.range as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiMutateGoalFuzzyLogic {
    fn decode_return(_word: isize) -> Self::Output {}
}
