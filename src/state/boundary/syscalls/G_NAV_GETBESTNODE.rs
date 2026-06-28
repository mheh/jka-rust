use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETBESTNODE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetbestnodeArgs {
    start_id: c_int,
    end_id: c_int,
    reject_id: c_int,
}

impl GNavGetbestnodeArgs {
    pub fn new(start_id: c_int, end_id: c_int, reject_id: c_int) -> Self {
        Self { start_id, end_id, reject_id }
    }

    pub fn start_id(&self) -> c_int { self.start_id }
    pub fn end_id(&self) -> c_int { self.end_id }
    pub fn reject_id(&self) -> c_int { self.reject_id }
}

pub struct GNavGetbestnode;

impl OutboundSysCall for GNavGetbestnode {
    type Args = GNavGetbestnodeArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_GETBESTNODE;
}

impl EncodeSysCall for GNavGetbestnode {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.start_id as isize,
            a.end_id as isize,
            a.reject_id as isize,
        ])
    }
}

impl DecodeSysCallReturn for GNavGetbestnode {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
