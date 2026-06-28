use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_CHECKALLFAILEDEDGES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavCheckallfailededgesArgs;

impl GNavCheckallfailededgesArgs {
    pub fn new() -> Self {
        Self
    }
}

pub struct GNavCheckallfailededges;

impl OutboundSysCall for GNavCheckallfailededges {
    type Import = GameImport;
    type Args = GNavCheckallfailededgesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_CHECKALLFAILEDEDGES;
}

impl EncodeSysCall for GNavCheckallfailededges {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavCheckallfailededges {
    fn decode_return(_word: isize) -> Self::Output {}
}
