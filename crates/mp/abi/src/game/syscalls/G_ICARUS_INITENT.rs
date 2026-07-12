use super::super::MpGameImport;
use mp_qshared::common::mp::gentity_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ICARUS_INITENT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GIcarusInitentArgs {
    ent: *mut gentity_t,
}

impl GIcarusInitentArgs {
    pub fn new(ent: *mut gentity_t) -> Self {
        Self { ent }
    }

    pub fn ent(&self) -> *mut gentity_t {
        self.ent
    }
}

/// `G_ICARUS_INITENT` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:261`
pub struct GIcarusInitent;

impl OutboundSysCall for GIcarusInitent {
    type Import = MpGameImport;
    type Args = GIcarusInitentArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_ICARUS_INITENT;
}

impl EncodeSysCall for GIcarusInitent {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.ent)])
    }
}

impl DecodeSysCallReturn for GIcarusInitent {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
