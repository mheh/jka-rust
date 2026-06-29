use core::ffi::c_int;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

/// `BOTLIB_AAS_INITIALIZED` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasInitializedArgs;

impl BotlibAasInitializedArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `BOTLIB_AAS_INITIALIZED` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:361`
pub struct BotlibAasInitialized;

impl OutboundSysCall for BotlibAasInitialized {
    type Import = GameImport;
    type Args = BotlibAasInitializedArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_INITIALIZED;
}

impl EncodeSysCall for BotlibAasInitialized {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for BotlibAasInitialized {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
