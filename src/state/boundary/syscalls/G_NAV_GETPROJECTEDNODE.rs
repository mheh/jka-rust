use core::ffi::c_int;

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;

use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_GETPROJECTEDNODE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavGetprojectednodeArgs {
    origin: *const vec3_t,
    node_id: i32,
}

impl GNavGetprojectednodeArgs {
    pub fn new(origin: *const vec3_t, node_id: i32) -> Self {
        Self { origin, node_id }
    }

    pub fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub fn node_id(&self) -> i32 {
        self.node_id
    }
}

pub struct GNavGetprojectednode;

impl OutboundSysCall for GNavGetprojectednode {
    type Args = GNavGetprojectednodeArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_NAV_GETPROJECTEDNODE;
}

impl EncodeSysCall for GNavGetprojectednode {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.origin()),
            a.node_id() as isize,
        ])
    }
}

impl DecodeSysCallReturn for GNavGetprojectednode {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
