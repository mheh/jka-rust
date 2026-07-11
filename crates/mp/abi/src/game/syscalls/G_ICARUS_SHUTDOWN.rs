use super::super::MpGameImport;

use abi_transport::generic::{
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

/// `G_ICARUS_SHUTDOWN` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:264`
pub struct GIcarusShutdown;

impl OutboundSysCall for GIcarusShutdown {
    type Import = MpGameImport;
    type Args = GIcarusShutdownArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_SHUTDOWN;
}

impl EncodeSysCall for GIcarusShutdown {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GIcarusShutdown {
    fn decode_return(_word: isize) -> Self::Output {}
}
