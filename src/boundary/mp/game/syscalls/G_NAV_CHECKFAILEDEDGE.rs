
use crate::codemp::game::g_public_h::failedEdge_t;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_CHECKFAILEDEDGE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavCheckfailededgeArgs {
    failed_edge: *mut failedEdge_t,
}

impl GNavCheckfailededgeArgs {
    pub fn new(failed_edge: *mut failedEdge_t) -> Self {
        Self { failed_edge }
    }

    pub fn failed_edge(&self) -> *mut failedEdge_t {
        self.failed_edge
    }
}

pub struct GNavCheckfailededge;

impl OutboundSysCall for GNavCheckfailededge {
    type Import = GameImport;
    type Args = GNavCheckfailededgeArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_NAV_CHECKFAILEDEDGE;
}

impl EncodeSysCall for GNavCheckfailededge {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.failed_edge)])
    }
}

impl DecodeSysCallReturn for GNavCheckfailededge {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
