use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;
use core::ffi::c_int;

/// `BOTLIB_SETUP` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibSetupArgs;

impl BotlibSetupArgs {
    pub fn new() -> Self {
        BotlibSetupArgs
    }
}

/// `BOTLIB_SETUP` MP game imports syscall ABI token.
///
/// Raven: ( void );
/// Source: `oracle/oracle/codemp/game/g_public.h:342`
pub struct BotlibSetup;

impl OutboundSysCall for BotlibSetup {
    type Import = GameImport;
    type Args = BotlibSetupArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_SETUP;
}

impl EncodeSysCall for BotlibSetup {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for BotlibSetup {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
