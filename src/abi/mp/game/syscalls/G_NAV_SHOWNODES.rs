use crate::ffi::GameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_SHOWNODES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavShownodesArgs;

impl GNavShownodesArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_NAV_SHOWNODES` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:305`
pub struct GNavShownodes;

impl OutboundSysCall for GNavShownodes {
    type Import = GameImport;
    type Args = GNavShownodesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_SHOWNODES;
}

impl EncodeSysCall for GNavShownodes {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavShownodes {
    fn decode_return(_word: isize) -> Self::Output {}
}
