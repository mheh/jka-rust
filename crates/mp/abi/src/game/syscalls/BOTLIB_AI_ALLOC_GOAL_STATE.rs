use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_ALLOC_GOAL_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiAllocGoalStateArgs {
    /// Client number / existing goal-state handle passed to the engine.
    state: c_int,
}

impl BotlibAiAllocGoalStateArgs {
    pub fn new(state: c_int) -> Self {
        Self { state }
    }

    pub fn state(&self) -> c_int {
        self.state
    }
}

/// `BOTLIB_AI_ALLOC_GOAL_STATE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:461`
pub struct BotlibAiAllocGoalState;

impl OutboundSysCall for BotlibAiAllocGoalState {
    type Import = MpGameImport;
    type Args = BotlibAiAllocGoalStateArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_ALLOC_GOAL_STATE;
}

impl EncodeSysCall for BotlibAiAllocGoalState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.state as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiAllocGoalState {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
