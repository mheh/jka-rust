use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

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

/// `G_NAV_GETNODENUMEDGES` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:311`
pub struct GNavGetnodenumedges;

impl OutboundSysCall for GNavGetnodenumedges {
    type Import = GameImport;
    type Args = GNavGetnodenumedgesArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_GETNODENUMEDGES;
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
