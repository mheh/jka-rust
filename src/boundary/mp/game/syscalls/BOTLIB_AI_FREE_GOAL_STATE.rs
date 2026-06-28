use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_FREE_GOAL_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiFreeGoalStateArgs {
    handle: c_int,
}

impl BotlibAiFreeGoalStateArgs {
    pub fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub fn handle(&self) -> c_int {
        self.handle
    }
}

pub struct BotlibAiFreeGoalState;

impl OutboundSysCall for BotlibAiFreeGoalState {
    type Import = GameImport;
    type Args = BotlibAiFreeGoalStateArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_FREE_GOAL_STATE;
}

impl EncodeSysCall for BotlibAiFreeGoalState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.handle as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiFreeGoalState {
    fn decode_return(_word: isize) -> Self::Output {}
}
