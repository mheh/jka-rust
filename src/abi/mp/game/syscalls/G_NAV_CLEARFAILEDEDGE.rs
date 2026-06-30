use super::super::MpGameImport;
use crate::common::mp::qcommon::failedEdge_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_CLEARFAILEDEDGE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavClearfailededgeArgs {
    failed_edge: *mut failedEdge_t,
}

impl GNavClearfailededgeArgs {
    pub fn new(failed_edge: *mut failedEdge_t) -> Self {
        Self { failed_edge }
    }

    pub fn failed_edge(&self) -> *mut failedEdge_t {
        self.failed_edge
    }
}

/// `G_NAV_CLEARFAILEDEDGE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:322`
pub struct GNavClearfailededge;

impl OutboundSysCall for GNavClearfailededge {
    type Import = MpGameImport;
    type Args = GNavClearfailededgeArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_CLEARFAILEDEDGE;
}

impl EncodeSysCall for GNavClearfailededge {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.failed_edge)])
    }
}

impl DecodeSysCallReturn for GNavClearfailededge {
    fn decode_return(_word: isize) -> Self::Output {}
}
