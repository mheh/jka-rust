use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::abi::sp::cgame::types::markFragment_t;
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_CM_MARKFRAGMENTS`.
///
/// Raven wrapper: `int trap_CM_MarkFragments(int numPoints, const vec3_t *points, const vec3_t projection, int maxPoints, vec3_t pointBuffer, int maxFragments, markFragment_t *fragmentBuffer);`
/// Raven transport: `return re.MarkFragments(args[1], (float(*)[3]) VMA(2), (const float *) VMA(3), args[4], (float *) VMA(5), args[6], (markFragment_t *) VMA(7));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:168-172`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:966`
/// Output source: `oracle/oracle/code/renderer/tr_public.h:102`
/// Buffer semantics source: `oracle/oracle/code/renderer/tr_marks.cpp:197-222`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:545-546`
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1402-1405`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmMarkfragmentsArgs {
    num_points: c_int,
    points: *const vec3_t,
    projection: *const vec3_t,
    max_points: c_int,
    point_buffer: *mut vec3_t,
    max_fragments: c_int,
    fragment_buffer: *mut markFragment_t,
}

impl CgCmMarkfragmentsArgs {
    pub const fn new(
        num_points: c_int,
        points: *const vec3_t,
        projection: *const vec3_t,
        max_points: c_int,
        point_buffer: *mut vec3_t,
        max_fragments: c_int,
        fragment_buffer: *mut markFragment_t,
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

    pub const fn fragment_buffer(&self) -> *mut markFragment_t {
        self.fragment_buffer
    }
}

/// `CG_CM_MARKFRAGMENTS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:89`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:168-172`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:545-546`
/// Output source: `oracle/oracle/code/renderer/tr_public.h:102`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:545-546`
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1402-1405`
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
