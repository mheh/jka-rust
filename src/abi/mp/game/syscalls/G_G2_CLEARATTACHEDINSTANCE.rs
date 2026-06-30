use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_G2_CLEARATTACHEDINSTANCE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GG2ClearattachedinstanceArgs {
    entity_num: c_int,
}

impl GG2ClearattachedinstanceArgs {
    pub fn new(entity_num: c_int) -> Self {
        Self { entity_num }
    }

    pub fn entity_num(&self) -> c_int {
        self.entity_num
    }
}

/// `G_G2_CLEARATTACHEDINSTANCE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:565`
pub struct GG2Clearattachedinstance;

impl OutboundSysCall for GG2Clearattachedinstance {
    type Import = MpGameImport;
    type Args = GG2ClearattachedinstanceArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_G2_CLEARATTACHEDINSTANCE;
}

impl EncodeSysCall for GG2Clearattachedinstance {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.entity_num as isize])
    }
}

impl DecodeSysCallReturn for GG2Clearattachedinstance {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
