use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_GETNODEEDGE` outbound game-to-engine syscall.
///
/// Returns the node ID of the `edge`-th neighbour of `node_id`.
#[derive(Debug)]
pub struct GNavGetnodeedgeArgs {
    node_id: c_int,
    edge: c_int,
}

impl GNavGetnodeedgeArgs {
    pub fn new(node_id: c_int, edge: c_int) -> Self {
        Self { node_id, edge }
    }

    pub fn node_id(&self) -> c_int {
        self.node_id
    }

    pub fn edge(&self) -> c_int {
        self.edge
    }
}

/// `G_NAV_GETNODEEDGE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:312`
pub struct GNavGetnodeedge;

impl OutboundSysCall for GNavGetnodeedge {
    type Import = MpGameImport;
    type Args = GNavGetnodeedgeArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_GETNODEEDGE;
}

impl EncodeSysCall for GNavGetnodeedge {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.node_id as isize, a.edge as isize])
    }
}

impl DecodeSysCallReturn for GNavGetnodeedge {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
