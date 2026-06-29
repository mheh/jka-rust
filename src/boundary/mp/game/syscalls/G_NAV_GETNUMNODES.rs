use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_GETNUMNODES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetnumnodesArgs;

impl GNavGetnumnodesArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_NAV_GETNUMNODES` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:313`
pub struct GNavGetnumnodes;

impl OutboundSysCall for GNavGetnumnodes {
    type Import = GameImport;
    type Args = GNavGetnumnodesArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_GETNUMNODES;
}

impl EncodeSysCall for GNavGetnumnodes {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for GNavGetnumnodes {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
