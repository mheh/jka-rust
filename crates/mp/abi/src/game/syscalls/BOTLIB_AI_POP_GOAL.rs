use super::super::MpGameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use core::ffi::c_int;

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

/// `BOTLIB_AI_POP_GOAL` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:443`
pub struct BotlibAiPopGoal;

impl OutboundSysCall for BotlibAiPopGoal {
    type Import = MpGameImport;
    type Args = BotlibAiPopGoalArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_POP_GOAL;
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
