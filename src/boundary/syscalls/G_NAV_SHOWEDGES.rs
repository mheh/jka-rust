use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_SHOWEDGES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavShowedgesArgs;

impl GNavShowedgesArgs {
    pub fn new() -> Self {
        Self
    }
}

pub struct GNavShowedges;

impl OutboundSysCall for GNavShowedges {
    type Args = GNavShowedgesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_SHOWEDGES;
}

impl EncodeSysCall for GNavShowedges {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavShowedges {
    fn decode_return(_word: isize) -> Self::Output {}
}
