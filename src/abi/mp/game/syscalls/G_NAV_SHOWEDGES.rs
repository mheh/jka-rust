use crate::ffi::GameImport;

use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_SHOWEDGES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavShowedgesArgs;

impl GNavShowedgesArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_NAV_SHOWEDGES` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:306`
pub struct GNavShowedges;

impl OutboundSysCall for GNavShowedges {
    type Import = GameImport;
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
