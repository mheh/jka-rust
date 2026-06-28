use core::ffi::c_int;
use crate::ffi::GameImport;
use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_FREE_MOVE_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiFreeMoveStateArgs {
    handle: c_int,
}

impl BotlibAiFreeMoveStateArgs {
    pub fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub fn handle(&self) -> c_int {
        self.handle
    }
}

pub struct BotlibAiFreeMoveState;

impl OutboundSysCall for BotlibAiFreeMoveState {
    type Args = BotlibAiFreeMoveStateArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_FREE_MOVE_STATE;
}

impl EncodeSysCall for BotlibAiFreeMoveState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.handle as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiFreeMoveState {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
