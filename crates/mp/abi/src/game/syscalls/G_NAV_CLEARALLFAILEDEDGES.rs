use super::super::MpGameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_CLEARALLFAILEDEDGES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavClearallfailededgesArgs;

impl GNavClearallfailededgesArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_NAV_CLEARALLFAILEDEDGES` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:323`
pub struct GNavClearallfailededges;

impl OutboundSysCall for GNavClearallfailededges {
    type Import = MpGameImport;
    type Args = GNavClearallfailededgesArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_CLEARALLFAILEDEDGES;
}

impl EncodeSysCall for GNavClearallfailededges {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavClearallfailededges {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
