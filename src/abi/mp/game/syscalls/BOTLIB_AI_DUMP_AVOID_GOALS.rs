use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_DUMP_AVOID_GOALS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiDumpAvoidGoalsArgs {
    goalstate: c_int,
}

impl BotlibAiDumpAvoidGoalsArgs {
    pub fn new(goalstate: c_int) -> Self {
        Self { goalstate }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }
}

/// `BOTLIB_AI_DUMP_AVOID_GOALS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:445`
pub struct BotlibAiDumpAvoidGoals;

impl OutboundSysCall for BotlibAiDumpAvoidGoals {
    type Import = MpGameImport;
    type Args = BotlibAiDumpAvoidGoalsArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_DUMP_AVOID_GOALS;
}

impl EncodeSysCall for BotlibAiDumpAvoidGoals {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiDumpAvoidGoals {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
