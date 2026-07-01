use super::super::MpGameImport;
use mp_qshared::common::mp::gentity_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

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

/// `G_ICARUS_ASSOCIATEENT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:263`
pub struct GIcarusAssociateent;

impl OutboundSysCall for GIcarusAssociateent {
    type Import = MpGameImport;
    type Args = GIcarusAssociateentArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_ASSOCIATEENT;
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
