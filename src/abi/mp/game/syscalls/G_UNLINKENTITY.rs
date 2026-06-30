use super::super::MpGameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::g_local::gentity_t;

/// `G_UNLINKENTITY` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GUnlinkentityArgs {
    ent: *mut gentity_t,
}

impl GUnlinkentityArgs {
    pub fn new(ent: *mut gentity_t) -> Self {
        Self { ent }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }
}

/// `G_UNLINKENTITY` MP game imports syscall ABI token.
///
/// Raven: ( gentity_t *ent );
/// Raven: call before removing an interactive entity
/// Source: `oracle/oracle/codemp/game/g_public.h:204`
pub struct GUnlinkentity;

impl OutboundSysCall for GUnlinkentity {
    type Import = MpGameImport;
    type Args = GUnlinkentityArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_UNLINKENTITY;
}

impl EncodeSysCall for GUnlinkentity {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent)])
    }
}

impl DecodeSysCallReturn for GUnlinkentity {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
