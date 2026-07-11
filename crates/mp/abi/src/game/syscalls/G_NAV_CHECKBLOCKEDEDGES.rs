use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_CHECKBLOCKEDEDGES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavCheckblockededgesArgs;

impl GNavCheckblockededgesArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_NAV_CHECKBLOCKEDEDGES` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:333`
pub struct GNavCheckblockededges;

impl OutboundSysCall for GNavCheckblockededges {
    type Import = MpGameImport;
    type Args = GNavCheckblockededgesArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_CHECKBLOCKEDEDGES;
}

impl EncodeSysCall for GNavCheckblockededges {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavCheckblockededges {
    fn decode_return(_word: isize) -> Self::Output {}
}
