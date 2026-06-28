use core::ffi::c_int;
use std::ffi::CString;

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AAS_VECTOR_FOR_BSP_EPAIR_KEY` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasVectorForBspEpairKeyArgs {
    ent: c_int,
    key: CString,
    v: *mut vec3_t,
}

impl BotlibAasVectorForBspEpairKeyArgs {
    pub fn new(ent: c_int, key: CString, v: *mut vec3_t) -> Self {
        Self { ent, key, v }
    }

    pub fn ent(&self) -> c_int {
        self.ent
    }

    pub fn key(&self) -> &CString {
        &self.key
    }

    pub fn v(&self) -> *mut vec3_t {
        self.v
    }
}

pub struct BotlibAasVectorForBspEpairKey;

impl OutboundSysCall for BotlibAasVectorForBspEpairKey {
    type Import = GameImport;
    type Args = BotlibAasVectorForBspEpairKeyArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_VECTOR_FOR_BSP_EPAIR_KEY;
}

impl EncodeSysCall for BotlibAasVectorForBspEpairKey {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.ent as isize,
            ptr_to_word(a.key.as_ptr()),
            ptr_to_word(a.v as *const vec3_t),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasVectorForBspEpairKey {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
