use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_CLEARCHECKEDNODES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavClearcheckednodesArgs;

impl GNavClearcheckednodesArgs {
    pub fn new() -> Self {
        Self
    }
}

pub struct GNavClearcheckednodes;

impl OutboundSysCall for GNavClearcheckednodes {
    type Import = GameImport;
    type Args = GNavClearcheckednodesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_CLEARCHECKEDNODES;
}

impl EncodeSysCall for GNavClearcheckednodes {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavClearcheckednodes {
    fn decode_return(_word: isize) -> Self::Output {}
}
