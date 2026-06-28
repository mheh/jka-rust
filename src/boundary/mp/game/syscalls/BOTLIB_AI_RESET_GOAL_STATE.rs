use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_RESET_GOAL_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiResetGoalStateArgs {
    goalstate: c_int,
}

impl BotlibAiResetGoalStateArgs {
    pub fn new(goalstate: c_int) -> Self {
        Self { goalstate }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }
}

pub struct BotlibAiResetGoalState;

impl OutboundSysCall for BotlibAiResetGoalState {
    type Import = GameImport;
    type Args = BotlibAiResetGoalStateArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_RESET_GOAL_STATE;
}

impl EncodeSysCall for BotlibAiResetGoalState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiResetGoalState {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
