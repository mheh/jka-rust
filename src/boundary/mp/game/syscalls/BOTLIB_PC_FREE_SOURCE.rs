use core::ffi::c_int;
use crate::ffi::GameImport;
use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Args for the `BOTLIB_PC_FREE_SOURCE` syscall.
///
/// Releases the precompiler source identified by `handle`.
#[derive(Debug)]
pub struct BotlibPcFreeSourceArgs {
    handle: c_int,
}

impl BotlibPcFreeSourceArgs {
    pub fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub fn handle(&self) -> c_int {
        self.handle
    }
}

pub struct BotlibPcFreeSource;

impl OutboundSysCall for BotlibPcFreeSource {
    type Import = GameImport;
    type Args = BotlibPcFreeSourceArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_PC_FREE_SOURCE;
}

impl EncodeSysCall for BotlibPcFreeSource {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.handle as isize])
    }
}

impl DecodeSysCallReturn for BotlibPcFreeSource {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
