use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::codemp::game::q_shared_h::vec3_t;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETNODEPOSITION` outbound game-to-engine syscall.
///
/// Fills `out` with the world-space position of the nav node identified by `node_id`.
/// Mirrors `syscall!(G_NAV_GETNODEPOSITION, node_id, out.as_mut_ptr())`.
#[derive(Debug)]
pub struct GNavGetnodepositionArgs {
    /// Nav-graph node identifier.
    node_id: c_int,
    /// Caller-allocated output buffer; engine writes the node's vec3 position here.
    out: *mut vec3_t,
}

impl GNavGetnodepositionArgs {
    pub fn new(node_id: c_int, out: *mut vec3_t) -> Self {
        Self { node_id, out }
    }

    pub fn node_id(&self) -> c_int {
        self.node_id
    }

    pub fn out(&self) -> *mut vec3_t {
        self.out
    }
}

pub struct GNavGetnodeposition;

impl OutboundSysCall for GNavGetnodeposition {
    type Args = GNavGetnodepositionArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_GETNODEPOSITION;
}

impl EncodeSysCall for GNavGetnodeposition {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.node_id as isize,
            ptr_to_word(a.out as *const u8),
        ])
    }
}

impl DecodeSysCallReturn for GNavGetnodeposition {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
