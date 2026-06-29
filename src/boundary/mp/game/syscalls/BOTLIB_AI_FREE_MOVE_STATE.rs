use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;
use core::ffi::c_int;

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

/// `BOTLIB_AI_FREE_MOVE_STATE` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:472`
pub struct BotlibAiFreeMoveState;

impl OutboundSysCall for BotlibAiFreeMoveState {
    type Import = GameImport;
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
