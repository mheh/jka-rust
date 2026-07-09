use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

/// `BOTLIB_AI_RESET_GOAL_STATE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:440`
pub struct BotlibAiResetGoalState;

impl OutboundSysCall for BotlibAiResetGoalState {
    type Import = MpGameImport;
    type Args = BotlibAiResetGoalStateArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_RESET_GOAL_STATE;
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
