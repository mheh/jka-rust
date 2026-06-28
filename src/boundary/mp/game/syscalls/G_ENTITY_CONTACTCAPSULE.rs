use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::types::qboolean;
use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ENTITY_CONTACTCAPSULE` outbound game-to-engine syscall.
///
/// Exact overlap test of the capsule `mins`..`maxs` against `ent`'s inline brush model.
#[derive(Debug)]
pub struct GEntityContactcapsuleArgs {
    mins: *const vec3_t,
    maxs: *const vec3_t,
    ent: *const gentity_t,
}

impl GEntityContactcapsuleArgs {
    pub fn new(mins: *const vec3_t, maxs: *const vec3_t, ent: *const gentity_t) -> Self {
        Self { mins, maxs, ent }
    }

    pub fn mins(&self) -> *const vec3_t {
        self.mins
    }

    pub fn maxs(&self) -> *const vec3_t {
        self.maxs
    }

    pub fn ent(&self) -> *const gentity_t {
        self.ent
    }
}

pub struct GEntityContactcapsule;

impl OutboundSysCall for GEntityContactcapsule {
    type Import = GameImport;
    type Args = GEntityContactcapsuleArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_ENTITY_CONTACTCAPSULE;
}

impl EncodeSysCall for GEntityContactcapsule {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.mins),
            ptr_to_word(a.maxs),
            ptr_to_word(a.ent),
        ])
    }
}

impl DecodeSysCallReturn for GEntityContactcapsule {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
