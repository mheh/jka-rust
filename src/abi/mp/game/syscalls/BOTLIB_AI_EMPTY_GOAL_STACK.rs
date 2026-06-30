use super::super::MpGameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use core::ffi::c_int;

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

/// `BOTLIB_AI_EMPTY_GOAL_STACK` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:444`
pub struct BotlibAiEmptyGoalStack;

impl OutboundSysCall for BotlibAiEmptyGoalStack {
    type Import = MpGameImport;
    type Args = BotlibAiEmptyGoalStackArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_EMPTY_GOAL_STACK;
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
