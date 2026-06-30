use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETNODENUMEDGES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetnodenumedgesArgs {
    node_id: c_int,
}

impl GNavGetnodenumedgesArgs {
    pub fn new(node_id: c_int) -> Self {
        Self { node_id }
    }

    pub fn node_id(&self) -> c_int {
        self.node_id
    }
}

/// `G_NAV_GETNODENUMEDGES` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:311`
pub struct GNavGetnodenumedges;

impl OutboundSysCall for GNavGetnodenumedges {
    type Import = MpGameImport;
    type Args = GNavGetnodenumedgesArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_GETNODENUMEDGES;
}

impl EncodeSysCall for GNavGetnodenumedges {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([(a.node_id as isize)])
    }
}

impl DecodeSysCallReturn for GNavGetnodenumedges {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
