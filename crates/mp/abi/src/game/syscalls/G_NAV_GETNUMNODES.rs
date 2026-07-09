use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETNUMNODES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetnumnodesArgs;

impl GNavGetnumnodesArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `G_NAV_GETNUMNODES` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:313`
pub struct GNavGetnumnodes;

impl OutboundSysCall for GNavGetnumnodes {
    type Import = MpGameImport;
    type Args = GNavGetnumnodesArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_GETNUMNODES;
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
