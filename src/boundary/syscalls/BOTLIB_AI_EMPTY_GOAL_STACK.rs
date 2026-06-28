use core::ffi::c_int;
use crate::ffi::GameImport;
use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_EMPTY_GOAL_STACK` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiEmptyGoalStackArgs {
    goalstate: c_int,
}

impl BotlibAiEmptyGoalStackArgs {
    pub fn new(goalstate: c_int) -> Self {
        Self { goalstate }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }
}

pub struct BotlibAiEmptyGoalStack;

impl OutboundSysCall for BotlibAiEmptyGoalStack {
    type Args = BotlibAiEmptyGoalStackArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_EMPTY_GOAL_STACK;
}

impl EncodeSysCall for BotlibAiEmptyGoalStack {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiEmptyGoalStack {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
