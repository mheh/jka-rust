use crate::codemp::game::g_local::gentity_t;
use crate::ffi::GameImport;

use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_NAV_CHECKFAILEDNODES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavCheckfailednodesArgs {
    ent: *mut gentity_t,
}

impl GNavCheckfailednodesArgs {
    pub fn new(ent: *mut gentity_t) -> Self {
        Self { ent }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }
}

pub struct GNavCheckfailednodes;

impl OutboundSysCall for GNavCheckfailednodes {
    type Args = GNavCheckfailednodesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_NAV_CHECKFAILEDNODES;
}

impl EncodeSysCall for GNavCheckfailednodes {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent)])
    }
}

impl DecodeSysCallReturn for GNavCheckfailednodes {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
