use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_CHECKBLOCKEDEDGES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavCheckblockededgesArgs;

impl GNavCheckblockededgesArgs {
    pub fn new() -> Self {
        Self
    }
}

pub struct GNavCheckblockededges;

impl OutboundSysCall for GNavCheckblockededges {
    type Args = GNavCheckblockededgesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_CHECKBLOCKEDEDGES;
}

impl EncodeSysCall for GNavCheckblockededges {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavCheckblockededges {
    fn decode_return(_word: isize) -> Self::Output {}
}
