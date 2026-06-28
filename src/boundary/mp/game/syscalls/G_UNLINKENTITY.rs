use crate::codemp::game::g_local::gentity_t;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct GUnlinkentity;

impl OutboundSysCall for GUnlinkentity {
    type Import = GameImport;
    type Args = GUnlinkentityArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_UNLINKENTITY;
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
