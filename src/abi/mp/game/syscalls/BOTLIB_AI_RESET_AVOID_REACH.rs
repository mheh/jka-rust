use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_RESET_AVOID_REACH` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiResetAvoidReachArgs {
    movestate: c_int,
}

impl BotlibAiResetAvoidReachArgs {
    pub fn new(movestate: c_int) -> Self {
        Self { movestate }
    }

    pub fn movestate(&self) -> c_int {
        self.movestate
    }
}

/// `BOTLIB_AI_RESET_AVOID_REACH` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:467`
pub struct BotlibAiResetAvoidReach;

impl OutboundSysCall for BotlibAiResetAvoidReach {
    type Import = GameImport;
    type Args = BotlibAiResetAvoidReachArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_RESET_AVOID_REACH;
}

impl EncodeSysCall for BotlibAiResetAvoidReach {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.movestate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiResetAvoidReach {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
