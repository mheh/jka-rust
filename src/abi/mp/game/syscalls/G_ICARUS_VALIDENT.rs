use super::super::MpGameImport;
use crate::common::mp::gentity_t;
use crate::shared::qboolean;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ICARUS_VALIDENT` outbound game-to-engine syscall.
///
/// Asks the engine whether `ent` has a live ICARUS instance.
/// Mirrors `syscall!(G_ICARUS_VALIDENT, ent) as qboolean`.
#[derive(Debug)]
pub struct GIcarusValidentArgs {
    /// Entity to query.
    pub ent: *mut gentity_t,
}

impl GIcarusValidentArgs {
    pub fn new(ent: *mut gentity_t) -> Self {
        Self { ent }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }
}

/// `G_ICARUS_VALIDENT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:256`
pub struct GIcarusValident;

impl OutboundSysCall for GIcarusValident {
    type Import = MpGameImport;
    type Args = GIcarusValidentArgs;
    type Output = qboolean;

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_VALIDENT;
}

impl EncodeSysCall for GIcarusValident {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent)])
    }
}

impl DecodeSysCallReturn for GIcarusValident {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
