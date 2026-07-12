use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_GETNODERADIUS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetnoderadiusArgs {
    node_id: c_int,
}

impl GNavGetnoderadiusArgs {
    pub fn new(node_id: c_int) -> Self {
        Self { node_id }
    }

    pub fn node_id(&self) -> c_int {
        self.node_id
    }
}

/// `G_NAV_GETNODERADIUS` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:332`
pub struct GNavGetnoderadius;

impl OutboundSysCall for GNavGetnoderadius {
    type Import = MpGameImport;
    type Args = GNavGetnoderadiusArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_GETNODERADIUS;
}

impl EncodeSysCall for GNavGetnoderadius {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.node_id as isize])
    }
}

impl DecodeSysCallReturn for GNavGetnoderadius {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
