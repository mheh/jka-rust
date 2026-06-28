use core::ffi::c_int;

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AAS_POINT_AREA_NUM` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasPointAreaNumArgs {
    point: *const vec3_t,
}

impl BotlibAasPointAreaNumArgs {
    pub fn new(point: *const vec3_t) -> Self {
        Self { point }
    }

    pub fn point(&self) -> *const vec3_t {
        self.point
    }
}

pub struct BotlibAasPointAreaNum;

impl OutboundSysCall for BotlibAasPointAreaNum {
    type Import = GameImport;
    type Args = BotlibAasPointAreaNumArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_POINT_AREA_NUM;
}

impl EncodeSysCall for BotlibAasPointAreaNum {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.point)])
    }
}

impl DecodeSysCallReturn for BotlibAasPointAreaNum {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
