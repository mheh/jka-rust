use super::super::MpGameImport;
use mp_qshared::common::mp::gentity_s;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_NAV_CHECKFAILEDNODES` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GNavCheckfailednodesArgs {
    ent: *mut gentity_s,
}

impl GNavCheckfailednodesArgs {
    pub fn new(ent: *mut gentity_s) -> Self {
        Self { ent }
    }

    pub fn ent(&self) -> *mut gentity_s {
        self.ent
    }
}

/// `G_NAV_CHECKFAILEDNODES` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:318`
pub struct GNavCheckfailednodes;

impl OutboundSysCall for GNavCheckfailednodes {
    type Import = MpGameImport;
    type Args = GNavCheckfailednodesArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_NAV_CHECKFAILEDNODES;
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
