use super::super::MpGameImport;
use mp_qshared::common::mp::gentity_t;
use mp_qshared::shared::qboolean;
use mp_qshared::shared::vec3_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ENTITY_CONTACT` outbound game-to-engine syscall.
///
/// C ABI: `qboolean trap_EntityContact(const vec3_t mins, const vec3_t maxs, const gentity_t *ent)`
///
/// Exact overlap test of the box `mins`..`maxs` against `ent`'s (possibly
/// non-axial) inline brush model.
#[derive(Debug)]
pub struct GEntityContactArgs {
    mins: *const vec3_t,
    maxs: *const vec3_t,
    ent: *const gentity_t,
}

impl GEntityContactArgs {
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

/// `G_ENTITY_CONTACT` MP game imports syscall ABI token.
///
/// Raven: ( const vec3_t mins, const vec3_t maxs, const gentity_t *ent );
/// Raven: perform an exact check against inline brush models of non-square shape
/// Raven: access for bots to get and free a server client (FIXME?)
/// Source: `oracle/oracle/codemp/game/g_public.h:211`
pub struct GEntityContact;

impl OutboundSysCall for GEntityContact {
    type Import = MpGameImport;
    type Args = GEntityContactArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_ENTITY_CONTACT;
}

impl EncodeSysCall for GEntityContact {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.mins as *const u8),
            ptr_to_word(a.maxs as *const u8),
            ptr_to_word(a.ent as *const u8),
        ])
    }
}

impl DecodeSysCallReturn for GEntityContact {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
