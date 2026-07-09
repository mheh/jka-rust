use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_CLEARCHECKEDNODES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavClearcheckednodesArgs;

impl GNavClearcheckednodesArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_NAV_CLEARCHECKEDNODES` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:334`
pub struct GNavClearcheckednodes;

impl OutboundSysCall for GNavClearcheckednodes {
    type Import = MpGameImport;
    type Args = GNavClearcheckednodesArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_CLEARCHECKEDNODES;
}

impl EncodeSysCall for GNavClearcheckednodes {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavClearcheckednodes {
    fn decode_return(_word: isize) -> Self::Output {}
}
