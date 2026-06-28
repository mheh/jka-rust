use core::ffi::c_int;
use crate::ffi::GameImport;
use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_POP_GOAL` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiPopGoalArgs {
    goalstate: c_int,
}

impl BotlibAiPopGoalArgs {
    pub fn new(goalstate: c_int) -> Self {
        Self { goalstate }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }
}

pub struct BotlibAiPopGoal;

impl OutboundSysCall for BotlibAiPopGoal {
    type Args = BotlibAiPopGoalArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_POP_GOAL;
}

impl EncodeSysCall for BotlibAiPopGoal {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiPopGoal {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
