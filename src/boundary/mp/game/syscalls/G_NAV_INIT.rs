use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_INIT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavInitArgs;

impl GNavInitArgs {
    pub fn new() -> Self {
        GNavInitArgs
    }
}

pub struct GNavInit;

impl OutboundSysCall for GNavInit {
    type Import = GameImport;
    type Args = GNavInitArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_INIT;
}

impl EncodeSysCall for GNavInit {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavInit {
    fn decode_return(_word: isize) -> Self::Output {}
}
