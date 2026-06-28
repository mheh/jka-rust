use core::ffi::c_int;

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct BotlibAasSwimming;

impl OutboundSysCall for BotlibAasSwimming {
    type Import = GameImport;
    type Args = BotlibAasSwimmingArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_SWIMMING;
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
