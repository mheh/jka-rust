use core::ffi::c_int;

use super::super::MpGameImport;
use mp_qshared::shared::qboolean;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_ROUTEBLOCKED` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavRouteblockedArgs {
    start_id: c_int,
    test_edge_id: c_int,
    end_id: c_int,
    reject_rank: c_int,
}

impl GNavRouteblockedArgs {
    pub fn new(start_id: c_int, test_edge_id: c_int, end_id: c_int, reject_rank: c_int) -> Self {
        Self {
            start_id,
            test_edge_id,
            end_id,
            reject_rank,
        }
    }

    pub fn start_id(&self) -> c_int {
        self.start_id
    }
    pub fn test_edge_id(&self) -> c_int {
        self.test_edge_id
    }
    pub fn end_id(&self) -> c_int {
        self.end_id
    }
    pub fn reject_rank(&self) -> c_int {
        self.reject_rank
    }
}

/// `G_NAV_ROUTEBLOCKED` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:328`
pub struct GNavRouteblocked;

impl OutboundSysCall for GNavRouteblocked {
    type Import = MpGameImport;
    type Args = GNavRouteblockedArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_NAV_ROUTEBLOCKED;
}

impl EncodeSysCall for GNavRouteblocked {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.start_id as isize,
            a.test_edge_id as isize,
            a.end_id as isize,
            a.reject_rank as isize,
        ])
    }
}

impl DecodeSysCallReturn for GNavRouteblocked {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
