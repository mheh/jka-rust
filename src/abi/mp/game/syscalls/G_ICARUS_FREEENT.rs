use crate::codemp::game::g_local::gentity_t;
use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ICARUS_FREEENT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GIcarusFreeentArgs {
    ent: *mut gentity_t,
}

impl GIcarusFreeentArgs {
    pub fn new(ent: *mut gentity_t) -> Self {
        Self { ent }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }
}

/// `G_ICARUS_FREEENT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:262`
pub struct GIcarusFreeent;

impl OutboundSysCall for GIcarusFreeent {
    type Import = GameImport;
    type Args = GIcarusFreeentArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_ICARUS_FREEENT;
}

impl EncodeSysCall for GIcarusFreeent {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent)])
    }
}

impl DecodeSysCallReturn for GIcarusFreeent {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
