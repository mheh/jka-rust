use super::super::MpGameImport;
use crate::common::mp::gentity_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `G_LINKENTITY`.
///
/// `ent` is the entity the engine makes visible to collision and to clients;
/// the engine reads through (and updates link fields on) this caller-owned
/// pointer, so it is held as a raw pointer rather than turned into a return.
#[derive(Debug)]
pub struct GLinkentityArgs {
    ent: *mut gentity_t,
}

impl GLinkentityArgs {
    pub const fn new(ent: *mut gentity_t) -> Self {
        Self { ent }
    }

    pub const fn ent(&self) -> *mut gentity_t {
        self.ent
    }
}

/// `G_LINKENTITY` MP game imports syscall ABI token.
///
/// Raven: ( gentity_t *ent );
/// Raven: an entity will never be sent to a client or used for collision
/// Raven: if it is not passed to linkentity.  If the size, position, or
/// Raven: solidity changes, it must be relinked.
/// Source: `oracle/oracle/codemp/game/g_public.h:199`
pub struct GLinkentity;

impl OutboundSysCall for GLinkentity {
    type Import = MpGameImport;
    type Args = GLinkentityArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_LINKENTITY;
}

impl EncodeSysCall for GLinkentity {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ent())])
    }
}

impl DecodeSysCallReturn for GLinkentity {
    // `trap_LinkEntity` is `void`; the engine's return word carries nothing.
    fn decode_return(_word: isize) -> Self::Output {}
}
