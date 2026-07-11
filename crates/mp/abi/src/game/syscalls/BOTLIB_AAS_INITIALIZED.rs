use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AAS_INITIALIZED` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasInitializedArgs;

impl BotlibAasInitializedArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `BOTLIB_AAS_INITIALIZED` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:361`
pub struct BotlibAasInitialized;

impl OutboundSysCall for BotlibAasInitialized {
    type Import = MpGameImport;
    type Args = BotlibAasInitializedArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AAS_INITIALIZED;
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
