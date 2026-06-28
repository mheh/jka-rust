use crate::ffi::GameImport;
use crate::ffi::types::qboolean;
use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETPATHSCALCULATED` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetpathscalculatedArgs;

impl GNavGetpathscalculatedArgs {
    pub fn new() -> Self {
        Self
    }
}

pub struct GNavGetpathscalculated;

impl OutboundSysCall for GNavGetpathscalculated {
    type Args = GNavGetpathscalculatedArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_NAV_GETPATHSCALCULATED;
}

impl EncodeSysCall for GNavGetpathscalculated {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavGetpathscalculated {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
