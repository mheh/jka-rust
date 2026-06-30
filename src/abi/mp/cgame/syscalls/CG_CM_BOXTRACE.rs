use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::common::mp::trace_t::trace_t;
use crate::shared::vec3_t;

/// Arguments for `CG_CM_BOXTRACE`.
///
/// C ABI: `void trap_CM_BoxTrace(trace_t *results, const vec3_t start,
/// const vec3_t end, const vec3_t mins, const vec3_t maxs, clipHandle_t model,
/// int brushmask)`.
///
/// Raven's wrapper forwards the writable trace result, four read-only vectors,
/// and two int-compatible collision words. The MP client switch decodes the
/// pointer-shaped words through `VMA`, calls `CM_BoxTrace(..., qfalse)`, writes
/// the trace through `results`, and returns `0`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:148-154`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2203-2205`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:793-795`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:793-795`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmBoxtraceArgs {
    /// Caller-owned trace output, decoded by Raven as `(trace_t *)VMA(1)`.
    results: *mut trace_t,
    /// Trace start vector, decoded by Raven as `(const float *)VMA(2)`.
    start: *const vec3_t,
    /// Trace end vector, decoded by Raven as `(const float *)VMA(3)`.
    end: *const vec3_t,
    /// Minimum bounds vector, decoded by Raven as `(const float *)VMA(4)`.
    mins: *const vec3_t,
    /// Maximum bounds vector, decoded by Raven as `(const float *)VMA(5)`.
    maxs: *const vec3_t,
    /// `clipHandle_t` model handle, passed as raw `args[6]`.
    model: c_int,
    /// Content mask passed as raw `args[7]`.
    brushmask: c_int,
}

impl CgCmBoxtraceArgs {
    /// Construct raw `trap_CM_BoxTrace` syscall args.
    ///
    /// # Safety
    /// `results` must be valid for a writable `trace_t`, and all vector
    /// pointers must be valid readable `vec3_t` values for the duration of the
    /// syscall.
    pub const unsafe fn new(
        results: *mut trace_t,
        start: *const vec3_t,
        end: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        model: c_int,
        brushmask: c_int,
    ) -> Self {
        Self {
            results,
            start,
            end,
            mins,
            maxs,
            model,
            brushmask,
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
}

/// `CG_CM_BOXTRACE` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_CM_BOXTRACE, results, start, end, mins, maxs, model, brushmask );`
/// Raven transport: `CM_BoxTrace((trace_t *)VMA(1), (const float *)VMA(2),
/// (const float *)VMA(3), (const float *)VMA(4), (const float *)VMA(5),
/// args[6], args[7], qfalse); return 0;`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:90`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:148-154`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2203-2205`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:793-795`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:793-795`
pub struct CgCmBoxtrace;

impl OutboundSysCall for CgCmBoxtrace {
    type Import = MpCgameImport;
    type Args = CgCmBoxtraceArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_BOXTRACE;
}

impl EncodeSysCall for CgCmBoxtrace {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.results()),
            ptr_to_word(args.start()),
            ptr_to_word(args.end()),
            ptr_to_word(args.mins()),
            ptr_to_word(args.maxs()),
            args.model() as isize,
            args.brushmask() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgCmBoxtrace {
    // Raven returns 0; the trace payload is written through the `results` out pointer.
    fn decode_return(_word: isize) -> Self::Output {}
}
