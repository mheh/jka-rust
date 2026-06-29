use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ICARUS_SHUTDOWN` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GIcarusShutdownArgs;

impl GIcarusShutdownArgs {
    pub fn new() -> Self {
        GIcarusShutdownArgs
    }
}

/// `G_ICARUS_SHUTDOWN` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:264`
pub struct GIcarusShutdown;

impl OutboundSysCall for GIcarusShutdown {
    type Import = GameImport;
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
