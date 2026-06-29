use core::ffi::{c_int, c_void};

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_CM_MARKFRAGMENTS`.
///
/// Raven wrapper: `int trap_CM_MarkFragments(int numPoints, const vec3_t *points, const vec3_t projection, int maxPoints, vec3_t pointBuffer, int maxFragments, markFragment_t *fragmentBuffer);`
/// Raven transport: `return re.MarkFragments(args[1], (float(*)[3]) VMA(2), (const float *) VMA(3), args[4], (float *) VMA(5), args[6], (markFragment_t *) VMA(7));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:168-172`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:966`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:545-546`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmMarkfragmentsArgs {
    num_points: c_int,
    points: *const vec3_t,
    projection: *const vec3_t,
    max_points: c_int,
    point_buffer: *mut vec3_t,
    max_fragments: c_int,
    /// FIXME: create type markFragment_t. Raven source: `oracle/oracle/code/game/q_shared.h:1405`.
    fragment_buffer: *mut c_void,
}

impl CgCmMarkfragmentsArgs {
    pub const fn new(
        num_points: c_int,
        points: *const vec3_t,
        projection: *const vec3_t,
        max_points: c_int,
        point_buffer: *mut vec3_t,
        max_fragments: c_int,
        fragment_buffer: *mut c_void,
    ) -> Self {
        Self {
            num_points,
            points,
            projection,
            max_points,
            point_buffer,
            max_fragments,
            fragment_buffer,
        }
    }

    pub const fn num_points(&self) -> c_int {
        self.num_points
    }

    pub const fn points(&self) -> *const vec3_t {
        self.points
    }

    pub const fn projection(&self) -> *const vec3_t {
        self.projection
    }

    pub const fn max_points(&self) -> c_int {
        self.max_points
    }

    pub const fn point_buffer(&self) -> *mut vec3_t {
        self.point_buffer
    }

    pub const fn max_fragments(&self) -> c_int {
        self.max_fragments
    }

    pub const fn fragment_buffer(&self) -> *mut c_void {
        self.fragment_buffer
    }
}

/// `CG_CM_MARKFRAGMENTS` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:89`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:168-172`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:545-546`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:545-546`
pub struct CgCmMarkfragments;

impl OutboundSysCall for CgCmMarkfragments {
    type Import = SpCgameImport;
    type Args = CgCmMarkfragmentsArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_MARKFRAGMENTS;
}

impl EncodeSysCall for CgCmMarkfragments {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.num_points() as isize,
            ptr_to_word(args.points()),
            ptr_to_word(args.projection()),
            args.max_points() as isize,
            ptr_to_word(args.point_buffer()),
            args.max_fragments() as isize,
            ptr_to_word(args.fragment_buffer()),
        ])
    }
}

impl DecodeSysCallReturn for CgCmMarkfragments {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
