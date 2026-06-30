use core::ffi::c_int;

use super::super::MpGameImport;
use crate::shared::qboolean;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_NODESARENEIGHBORS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavNodesareneighborsArgs {
    start_id: c_int,
    end_id: c_int,
}

impl GNavNodesareneighborsArgs {
    pub fn new(start_id: c_int, end_id: c_int) -> Self {
        Self { start_id, end_id }
    }

    pub fn start_id(&self) -> c_int {
        self.start_id
    }

    pub fn end_id(&self) -> c_int {
        self.end_id
    }
}

/// `G_NAV_NODESARENEIGHBORS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:321`
pub struct GNavNodesareneighbors;

impl OutboundSysCall for GNavNodesareneighbors {
    type Import = MpGameImport;
    type Args = GNavNodesareneighborsArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_NODESARENEIGHBORS;
}

impl EncodeSysCall for GNavNodesareneighbors {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.start_id as isize, a.end_id as isize])
    }
}

impl DecodeSysCallReturn for GNavNodesareneighbors {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
