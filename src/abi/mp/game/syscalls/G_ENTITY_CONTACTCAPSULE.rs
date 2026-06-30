use super::super::MpGameImport;
use crate::common::mp::gentity_t;
use crate::shared::qboolean;
use crate::shared::vec3_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

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

/// `G_ENTITY_CONTACTCAPSULE` MP game imports syscall ABI token.
///
/// Raven: ( const vec3_t mins, const vec3_t maxs, const gentity_t *ent );
/// Raven: SP_REGISTER_SERVER_CMD,
/// Source: `oracle/oracle/codemp/game/g_public.h:236`
pub struct GEntityContactcapsule;

impl OutboundSysCall for GEntityContactcapsule {
    type Import = MpGameImport;
    type Args = GEntityContactcapsuleArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_ENTITY_CONTACTCAPSULE;
}

impl EncodeSysCall for GEntityContactcapsule {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.mins), ptr_to_word(a.maxs), ptr_to_word(a.ent)])
    }
}

impl DecodeSysCallReturn for GEntityContactcapsule {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
