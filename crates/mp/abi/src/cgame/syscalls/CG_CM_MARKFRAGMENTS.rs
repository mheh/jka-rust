#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// `markFragment_t` ABI record returned through `fragmentBuffer`.
///
/// Raven source: `oracle/oracle/codemp/game/q_shared.h:1918-1922`
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct markFragment_t {
    pub firstPoint: c_int,
    pub numPoints: c_int,
}

/// Arguments for `CG_CM_MARKFRAGMENTS`.
///
/// Raven wrapper: `int trap_CM_MarkFragments(int numPoints,
/// const vec3_t *points, const vec3_t projection, int maxPoints,
/// vec3_t pointBuffer, int maxFragments, markFragment_t *fragmentBuffer)`.
/// The MP client switch decodes `points`, `projection`, `pointBuffer`, and
/// `fragmentBuffer` through `VMA`; `pointBuffer` is declared as `vec3_t` in C
/// parameter position, so it is transported as a writable `float *` backing
/// `maxPoints` packed `vec3_t` vertices.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:177-181`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2212-2215`
/// Buffer semantics source: `oracle/oracle/codemp/renderer/tr_marks.cpp:189-223`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:805-806`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmMarkfragmentsArgs {
    /// Number of input polygon points, read by Raven as raw `args[1]`.
    num_points: c_int,
    /// Input polygon points, decoded by Raven as `(const vec3_t *)VMA(2)`.
    points: *const vec3_t,
    /// Projection vector, decoded by Raven as `(const float *)VMA(3)`.
    projection: *const vec3_t,
    /// Capacity of the packed output point buffer, read as raw `args[4]`.
    max_points: c_int,
    /// Writable packed output vertices, decoded by Raven as `(float *)VMA(5)`.
    point_buffer: *mut vec3_t,
    /// Capacity of `fragmentBuffer`, read as raw `args[6]`.
    max_fragments: c_int,
    /// Writable fragment records, decoded by Raven as `(markFragment_t *)VMA(7)`.
    fragment_buffer: *mut markFragment_t,
}

impl CgCmMarkfragmentsArgs {
    /// Construct raw `trap_CM_MarkFragments` syscall args.
    ///
    /// # Safety
    /// `points` and `projection` must be valid for reads. `point_buffer` must
    /// be writable for `max_points` packed `vec3_t` values, and
    /// `fragment_buffer` must be writable for `max_fragments` `markFragment_t`
    /// records for the duration of the syscall.
    pub const unsafe fn new(
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

/// `CG_CM_MARKFRAGMENTS` MP cgame imports syscall ABI token.
///
/// Raven comment: "Returns the projection of a polygon onto the solid brushes
/// in the world".
/// Raven wrapper: `return syscall( CG_CM_MARKFRAGMENTS, numPoints, points,
/// projection, maxPoints, pointBuffer, maxFragments, fragmentBuffer );`
/// Raven transport: `return re.MarkFragments(args[1], (const vec3_t *)VMA(2),
/// (const float *)VMA(3), args[4], (float *)VMA(5), args[6],
/// (markFragment_t *)VMA(7));`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:94`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:177-181`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2212-2215`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:177-181`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:805-806`
/// Output source: `oracle/oracle/codemp/renderer/tr_public.h:83-84`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:805-806`
pub struct CgCmMarkfragments;

impl OutboundSysCall for CgCmMarkfragments {
    type Import = MpCgameImport;
    type Args = CgCmMarkfragmentsArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_MARKFRAGMENTS;
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
    // `re.MarkFragments` returns the number of fragments written to `fragmentBuffer`.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
