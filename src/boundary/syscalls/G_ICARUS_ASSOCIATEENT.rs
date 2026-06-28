use crate::codemp::game::g_local::gentity_t;
use crate::ffi::GameImport;

use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_ASSOCIATEENT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GIcarusAssociateentArgs {
    ent: *mut gentity_t,
}

impl GIcarusAssociateentArgs {
    pub fn new(ent: *mut gentity_t) -> Self {
        Self { ent }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }
}

pub struct GIcarusAssociateent;

impl OutboundSysCall for GIcarusAssociateent {
    type Args = GIcarusAssociateentArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_ICARUS_ASSOCIATEENT;
}

impl EncodeSysCall for GIcarusAssociateent {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent)])
    }
}

impl DecodeSysCallReturn for GIcarusAssociateent {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
