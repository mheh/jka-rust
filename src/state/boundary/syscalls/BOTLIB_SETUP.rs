use core::ffi::c_int;
use crate::ffi::GameImport;
use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_SETUP` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibSetupArgs;

impl BotlibSetupArgs {
    pub fn new() -> Self {
        BotlibSetupArgs
    }
}

pub struct BotlibSetup;

impl OutboundSysCall for BotlibSetup {
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
