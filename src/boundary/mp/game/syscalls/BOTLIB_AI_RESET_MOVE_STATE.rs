use core::ffi::c_int;
use crate::ffi::GameImport;
use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_RESET_MOVE_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiResetMoveStateArgs {
    movestate: c_int,
}

impl BotlibAiResetMoveStateArgs {
    pub fn new(movestate: c_int) -> Self {
        Self { movestate }
    }

    pub fn movestate(&self) -> c_int {
        self.movestate
    }
}

pub struct BotlibAiResetMoveState;

impl OutboundSysCall for BotlibAiResetMoveState {
    type Import = GameImport;
    type Args = BotlibAiResetMoveStateArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_RESET_MOVE_STATE;
}

impl EncodeSysCall for BotlibAiResetMoveState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.movestate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiResetMoveState {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
