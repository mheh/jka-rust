use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_FREE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavFreeArgs;

impl GNavFreeArgs {
    pub fn new() -> Self {
        GNavFreeArgs
    }
}

/// `G_NAV_FREE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:299`
pub struct GNavFree;

impl OutboundSysCall for GNavFree {
    type Import = MpGameImport;
    type Args = GNavFreeArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_FREE;
}

impl EncodeSysCall for GNavFree {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavFree {
    fn decode_return(_word: isize) -> Self::Output {}
}
