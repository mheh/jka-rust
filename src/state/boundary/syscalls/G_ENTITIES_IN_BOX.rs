use core::ffi::c_int;

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;

use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ENTITIES_IN_BOX` outbound game-to-engine syscall.
///
/// C ABI: `int trap_EntitiesInBox(const vec3_t mins, const vec3_t maxs, int *list, int maxcount)`
#[derive(Debug)]
pub struct GEntitiesInBoxArgs {
    mins: *const vec3_t,
    maxs: *const vec3_t,
    list: *mut c_int,
    maxcount: c_int,
}

impl GEntitiesInBoxArgs {
    pub fn new(mins: *const vec3_t, maxs: *const vec3_t, list: *mut c_int, maxcount: c_int) -> Self {
        Self { mins, maxs, list, maxcount }
    }

    pub fn mins(&self) -> *const vec3_t { self.mins }
    pub fn maxs(&self) -> *const vec3_t { self.maxs }
    pub fn list(&self) -> *mut c_int { self.list }
    pub fn maxcount(&self) -> c_int { self.maxcount }
}

pub struct GEntitiesInBox;

impl OutboundSysCall for GEntitiesInBox {
    type Args = GEntitiesInBoxArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_ENTITIES_IN_BOX;
}

impl EncodeSysCall for GEntitiesInBox {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.mins as *const u8),
            ptr_to_word(a.maxs as *const u8),
            ptr_to_word(a.list as *const u8),
            a.maxcount as isize,
        ])
    }
}

impl DecodeSysCallReturn for GEntitiesInBox {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
