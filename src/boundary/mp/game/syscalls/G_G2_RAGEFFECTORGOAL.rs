use core::ffi::c_void;
use std::ffi::CString;

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_G2_RAGEFFECTORGOAL` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2RageffectorgoalArgs {
    ghoul2: *mut c_void,
    bone_name: CString,
    pos: *const vec3_t,
}

impl GG2RageffectorgoalArgs {
    pub fn new(ghoul2: *mut c_void, bone_name: CString, pos: *const vec3_t) -> Self {
        Self { ghoul2, bone_name, pos }
    }

    pub fn ghoul2(&self) -> *mut c_void {
        self.ghoul2
    }

    pub fn bone_name(&self) -> &CString {
        &self.bone_name
    }

    pub fn pos(&self) -> *const vec3_t {
        self.pos
    }
}

pub struct GG2Rageffectorgoal;

impl OutboundSysCall for GG2Rageffectorgoal {
    type Import = GameImport;
    type Args = GG2RageffectorgoalArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_G2_RAGEFFECTORGOAL;
}

impl EncodeSysCall for GG2Rageffectorgoal {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.ghoul2),
            ptr_to_word(a.bone_name.as_ptr()),
            ptr_to_word(a.pos),
        ])
    }
}

impl DecodeSysCallReturn for GG2Rageffectorgoal {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
