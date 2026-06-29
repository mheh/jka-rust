use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::{trace_t, vec3_t};

/// Arguments for `CG_CM_TRANSFORMEDCAPSULETRACE`.
///
/// Raven wrapper: `void trap_CM_TransformedCapsuleTrace(trace_t *results,
/// const vec3_t start, const vec3_t end, const vec3_t mins, const vec3_t maxs,
/// clipHandle_t model, int brushmask, const vec3_t origin,
/// const vec3_t angles)`.
///
/// The client switch decodes `results`, the six `vec3_t` pointer operands, and
/// the two integer operands, then calls `CM_TransformedBoxTrace(..., qtrue)`.
/// The trace is written through `results`; the syscall return word is always 0.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:171-174`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:171-174`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:802-804`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmTransformedcapsuletraceArgs {
    /// Output trace buffer, decoded by Raven as `(trace_t *)VMA(1)`.
    results: *mut trace_t,
    /// Trace start vector, decoded by Raven as `(const float *)VMA(2)`.
    start: *const vec3_t,
    /// Trace end vector, decoded by Raven as `(const float *)VMA(3)`.
    end: *const vec3_t,
    /// Minimum bounds vector, decoded by Raven as `(const float *)VMA(4)`.
    mins: *const vec3_t,
    /// Maximum bounds vector, decoded by Raven as `(const float *)VMA(5)`.
    maxs: *const vec3_t,
    /// `clipHandle_t` model handle, transported as an integer word.
    model: c_int,
    /// Brush/content mask, transported as an integer word.
    brushmask: c_int,
    /// Transform origin vector, decoded by Raven as `(const float *)VMA(8)`.
    origin: *const vec3_t,
    /// Transform angles vector, decoded by Raven as `(const float *)VMA(9)`.
    angles: *const vec3_t,
}

impl CgCmTransformedcapsuletraceArgs {
    pub const fn new(
        results: *mut trace_t,
        start: *const vec3_t,
        end: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        model: c_int,
        brushmask: c_int,
        origin: *const vec3_t,
        angles: *const vec3_t,
    ) -> Self {
        Self {
            results,
            start,
            end,
            mins,
            maxs,
            model,
            brushmask,
            origin,
            angles,
        }
    }

    pub const fn results(&self) -> *mut trace_t {
        self.results
    }

    pub const fn start(&self) -> *const vec3_t {
        self.start
    }

    pub const fn end(&self) -> *const vec3_t {
        self.end
    }

    pub const fn mins(&self) -> *const vec3_t {
        self.mins
    }

    pub const fn maxs(&self) -> *const vec3_t {
        self.maxs
    }

    pub const fn model(&self) -> c_int {
        self.model
    }

    pub const fn brushmask(&self) -> c_int {
        self.brushmask
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub const fn angles(&self) -> *const vec3_t {
        self.angles
    }
}

/// `CG_CM_TRANSFORMEDCAPSULETRACE` MP cgame imports syscall boundary token.
///
/// Raven wrapper: `syscall( CG_CM_TRANSFORMEDCAPSULETRACE, results, start, end,
/// mins, maxs, model, brushmask, origin, angles );`
/// Raven transport: `CM_TransformedBoxTrace(..., /*int capsule*/ qtrue);
/// return 0;`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:93`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:171-174`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:171-174`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:802-804`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:802-804`
pub struct CgCmTransformedcapsuletrace;

impl OutboundSysCall for CgCmTransformedcapsuletrace {
    type Import = MpCgameImport;
    type Args = CgCmTransformedcapsuletraceArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_TRANSFORMEDCAPSULETRACE;
}

impl EncodeSysCall for CgCmTransformedcapsuletrace {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.results()),
            ptr_to_word(args.start()),
            ptr_to_word(args.end()),
            ptr_to_word(args.mins()),
            ptr_to_word(args.maxs()),
            args.model() as isize,
            args.brushmask() as isize,
            ptr_to_word(args.origin()),
            ptr_to_word(args.angles()),
        ])
    }
}

impl DecodeSysCallReturn for CgCmTransformedcapsuletrace {
    // Raven returns 0; the actual `trace_t` result is written through `results`.
    fn decode_return(_word: isize) -> Self::Output {}
}
