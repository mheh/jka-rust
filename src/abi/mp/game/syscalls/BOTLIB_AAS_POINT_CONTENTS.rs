use core::ffi::c_int;

use super::super::MpGameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::vec3_t;

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

/// `BOTLIB_AAS_POINT_CONTENTS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:368`
pub struct BotlibAasPointContents;

impl OutboundSysCall for BotlibAasPointContents {
    type Import = MpGameImport;
    type Args = BotlibAasPointContentsArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AAS_POINT_CONTENTS;
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
