use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_CHECKALLFAILEDEDGES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavCheckallfailededgesArgs;

impl GNavCheckallfailededgesArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_NAV_CHECKALLFAILEDEDGES` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:327`
pub struct GNavCheckallfailededges;

impl OutboundSysCall for GNavCheckallfailededges {
    type Import = MpGameImport;
    type Args = GNavCheckallfailededgesArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_CHECKALLFAILEDEDGES;
}

impl EncodeSysCall for GNavCheckallfailededges {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavCheckallfailededges {
    fn decode_return(_word: isize) -> Self::Output {}
}
