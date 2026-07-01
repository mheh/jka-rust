use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// `G_POINT_CONTENTS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GPointContentsArgs {
    point: *const vec3_t,
    pass_entity_num: c_int,
}

impl GPointContentsArgs {
    pub fn new(point: *const vec3_t, pass_entity_num: c_int) -> Self {
        Self {
            point,
            pass_entity_num,
        }
    }

    pub fn point(&self) -> *const vec3_t {
        self.point
    }

    pub fn pass_entity_num(&self) -> c_int {
        self.pass_entity_num
    }
}

/// `G_POINT_CONTENTS` MP game imports syscall ABI token.
///
/// Raven: ( const vec3_t point, int passEntityNum );
/// Raven: point contents against all linked entities
/// Source: `oracle/oracle/codemp/game/g_public.h:188`
pub struct GPointContents;

impl OutboundSysCall for GPointContents {
    type Import = MpGameImport;
    type Args = GPointContentsArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_POINT_CONTENTS;
}

impl EncodeSysCall for GPointContents {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.point), a.pass_entity_num as isize])
    }
}

impl DecodeSysCallReturn for GPointContents {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
