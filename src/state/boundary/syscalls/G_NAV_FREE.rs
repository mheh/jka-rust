use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_FREE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavFreeArgs;

impl GNavFreeArgs {
    pub fn new() -> Self {
        GNavFreeArgs
    }
}

pub struct GNavFree;

impl OutboundSysCall for GNavFree {
    type Args = GNavFreeArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_FREE;
}

impl EncodeSysCall for GNavFree {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavFree {
    fn decode_return(_word: isize) -> Self::Output {}
}
