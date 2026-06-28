use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_SHUTDOWN` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GIcarusShutdownArgs;

impl GIcarusShutdownArgs {
    pub fn new() -> Self {
        GIcarusShutdownArgs
    }
}

pub struct GIcarusShutdown;

impl OutboundSysCall for GIcarusShutdown {
    type Args = GIcarusShutdownArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_ICARUS_SHUTDOWN;
}

impl EncodeSysCall for GIcarusShutdown {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GIcarusShutdown {
    fn decode_return(_word: isize) -> Self::Output {}
}
