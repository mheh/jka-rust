use core::ffi::c_int;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;

/// `G_DEBUG_POLYGON_CREATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GDebugPolygonCreateArgs {
    pub color: c_int,
    pub num_points: c_int,
    pub points: *mut vec3_t,
}

impl GDebugPolygonCreateArgs {
    pub fn new(color: c_int, num_points: c_int, points: *mut vec3_t) -> Self {
        Self {
            color,
            num_points,
            points,
        }
    }

    pub fn color(&self) -> c_int {
        self.color
    }
    pub fn num_points(&self) -> c_int {
        self.num_points
    }
    pub fn points(&self) -> *mut vec3_t {
        self.points
    }
}

/// `G_DEBUG_POLYGON_CREATE` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:230`
pub struct GDebugPolygonCreate;

impl OutboundSysCall for GDebugPolygonCreate {
    type Import = GameImport;
    type Args = GDebugPolygonCreateArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_DEBUG_POLYGON_CREATE;
}

impl EncodeSysCall for GDebugPolygonCreate {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.color as isize,
            a.num_points as isize,
            ptr_to_word(a.points),
        ])
    }
}

impl DecodeSysCallReturn for GDebugPolygonCreate {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
