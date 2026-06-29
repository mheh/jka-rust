use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_GETBESTNODEALTROUTE` outbound game-to-engine syscall.
///
/// C signature: `int trap_Nav_GetBestNodeAltRoute(int startID, int endID, int *pathCost, int rejectID)`
#[derive(Debug)]
/// `G_NAV_GETBESTNODEALTROUTE` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:329`
pub struct GNavGetbestnodealtroute;

#[derive(Debug)]
pub struct GNavGetbestnodealtRouteArgs {
    start_id: c_int,
    end_id: c_int,
    path_cost: *mut c_int,
    reject_id: c_int,
}

impl GNavGetbestnodealtRouteArgs {
    pub fn new(start_id: c_int, end_id: c_int, path_cost: *mut c_int, reject_id: c_int) -> Self {
        Self {
            start_id,
            end_id,
            path_cost,
            reject_id,
        }
    }

    pub fn start_id(&self) -> c_int {
        self.start_id
    }
    pub fn end_id(&self) -> c_int {
        self.end_id
    }
    pub fn path_cost(&self) -> *mut c_int {
        self.path_cost
    }
    pub fn reject_id(&self) -> c_int {
        self.reject_id
    }
}

impl OutboundSysCall for GNavGetbestnodealtroute {
    type Import = GameImport;
    type Args = GNavGetbestnodealtRouteArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_GETBESTNODEALTROUTE;
}

impl EncodeSysCall for GNavGetbestnodealtroute {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.start_id as isize,
            a.end_id as isize,
            ptr_to_word(a.path_cost),
            a.reject_id as isize,
        ])
    }
}

impl DecodeSysCallReturn for GNavGetbestnodealtroute {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
