use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct BotlibAiDumpAvoidGoals;

impl OutboundSysCall for BotlibAiDumpAvoidGoals {
    type Import = GameImport;
    type Args = BotlibAiDumpAvoidGoalsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_DUMP_AVOID_GOALS;
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
