use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// `BOTLIB_AAS_SWIMMING` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasSwimmingArgs {
    origin: *const vec3_t,
}

impl BotlibAasSwimmingArgs {
    pub fn new(origin: *const vec3_t) -> Self {
        Self { origin }
    }

    pub fn origin(&self) -> *const vec3_t {
        self.origin
    }
}

/// `BOTLIB_AAS_SWIMMING` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:379`
pub struct BotlibAasSwimming;

impl OutboundSysCall for BotlibAasSwimming {
    type Import = MpGameImport;
    type Args = BotlibAasSwimmingArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AAS_SWIMMING;
}

impl EncodeSysCall for BotlibAasSwimming {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.origin)])
    }
}

impl DecodeSysCallReturn for BotlibAasSwimming {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
