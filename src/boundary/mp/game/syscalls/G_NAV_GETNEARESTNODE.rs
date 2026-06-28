use core::ffi::c_int;

use crate::codemp::game::g_local::gentity_t;
use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETNEARESTNODE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetnearestnodeArgs {
    ent: *mut gentity_t,
    last_id: c_int,
    flags: c_int,
    target_id: c_int,
}

impl GNavGetnearestnodeArgs {
    pub fn new(ent: *mut gentity_t, last_id: c_int, flags: c_int, target_id: c_int) -> Self {
        Self { ent, last_id, flags, target_id }
    }

    pub fn ent(&self) -> *mut gentity_t { self.ent }
    pub fn last_id(&self) -> c_int { self.last_id }
    pub fn flags(&self) -> c_int { self.flags }
    pub fn target_id(&self) -> c_int { self.target_id }
}

pub struct GNavGetnearestnode;

impl OutboundSysCall for GNavGetnearestnode {
    type Import = GameImport;
    type Args = GNavGetnearestnodeArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_GETNEARESTNODE;
}

impl EncodeSysCall for GNavGetnearestnode {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ent),
            a.last_id as isize,
            a.flags as isize,
            a.target_id as isize,
        ])
    }
}

impl DecodeSysCallReturn for GNavGetnearestnode {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
