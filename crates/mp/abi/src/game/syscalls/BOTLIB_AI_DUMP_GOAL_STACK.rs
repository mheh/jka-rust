use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_DUMP_GOAL_STACK` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiDumpGoalStackArgs {
    goalstate: c_int,
}

impl BotlibAiDumpGoalStackArgs {
    pub fn new(goalstate: c_int) -> Self {
        Self { goalstate }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }
}

/// `BOTLIB_AI_DUMP_GOAL_STACK` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:446`
pub struct BotlibAiDumpGoalStack;

impl OutboundSysCall for BotlibAiDumpGoalStack {
    type Import = MpGameImport;
    type Args = BotlibAiDumpGoalStackArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_DUMP_GOAL_STACK;
}

impl EncodeSysCall for BotlibAiDumpGoalStack {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiDumpGoalStack {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
