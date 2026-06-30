use core::ffi::c_int;

use super::super::MpGameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_SHUTDOWN` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibShutdownArgs;

impl BotlibShutdownArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `BOTLIB_SHUTDOWN` MP game imports syscall ABI token.
///
/// Raven: ( void );
/// Source: `oracle/oracle/codemp/game/g_public.h:343`
pub struct BotlibShutdown;

impl OutboundSysCall for BotlibShutdown {
    type Import = MpGameImport;
    type Args = BotlibShutdownArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_SHUTDOWN;
}

impl EncodeSysCall for BotlibShutdown {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for BotlibShutdown {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
