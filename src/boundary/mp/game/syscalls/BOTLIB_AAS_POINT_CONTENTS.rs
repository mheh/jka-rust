use core::ffi::c_int;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;

/// `BOTLIB_AAS_POINT_CONTENTS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasPointContentsArgs {
    point: *const vec3_t,
}

impl BotlibAasPointContentsArgs {
    pub fn new(point: *const vec3_t) -> Self {
        Self { point }
    }

    pub fn point(&self) -> *const vec3_t {
        self.point
    }
}

/// `BOTLIB_AAS_POINT_CONTENTS` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:368`
pub struct BotlibAasPointContents;

impl OutboundSysCall for BotlibAasPointContents {
    type Import = GameImport;
    type Args = BotlibAasPointContentsArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_POINT_CONTENTS;
}

impl EncodeSysCall for BotlibAasPointContents {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.point)])
    }
}

impl DecodeSysCallReturn for BotlibAasPointContents {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
