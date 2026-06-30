use core::ffi::c_int;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::GameImport;

/// `BOTLIB_AI_RESET_LAST_AVOID_REACH` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiResetLastAvoidReachArgs {
    movestate: c_int,
}

impl BotlibAiResetLastAvoidReachArgs {
    pub fn new(movestate: c_int) -> Self {
        Self { movestate }
    }

    pub fn movestate(&self) -> c_int {
        self.movestate
    }
}

/// `BOTLIB_AI_RESET_LAST_AVOID_REACH` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:468`
pub struct BotlibAiResetLastAvoidReach;

impl OutboundSysCall for BotlibAiResetLastAvoidReach {
    type Import = GameImport;
    type Args = BotlibAiResetLastAvoidReachArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_RESET_LAST_AVOID_REACH;
}

impl EncodeSysCall for BotlibAiResetLastAvoidReach {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.movestate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiResetLastAvoidReach {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
