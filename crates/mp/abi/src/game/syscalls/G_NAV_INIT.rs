use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_INIT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavInitArgs;

impl GNavInitArgs {
    pub fn new() -> Self {
        GNavInitArgs
    }
}

/// `G_NAV_INIT` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:298`
pub struct GNavInit;

impl OutboundSysCall for GNavInit {
    type Import = MpGameImport;
    type Args = GNavInitArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_INIT;
}

impl EncodeSysCall for GNavInit {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavInit {
    fn decode_return(_word: isize) -> Self::Output {}
}
