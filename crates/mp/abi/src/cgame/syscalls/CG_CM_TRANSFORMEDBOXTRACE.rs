use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_CM_TRANSFORMEDBOXTRACE`.
///
/// Raven wrapper: `void trap_CM_TransformedBoxTrace(trace_t *results,
/// const vec3_t start, const vec3_t end, const vec3_t mins,
/// const vec3_t maxs, clipHandle_t model, int brushmask,
/// const vec3_t origin, const vec3_t angles)`.
/// The client switch decodes pointer arguments with `VMA`, reads `model` and
/// `brushmask` directly from `args[6]`/`args[7]`, and passes `qfalse` for the
/// non-capsule trace flag.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:163-167`
/// Args declaration source: `oracle/oracle/codemp/cgame/cg_local.h:2206-2209`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:799-801`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmTransformedboxtraceArgs {
    /// Caller-owned trace output, decoded by Raven as `(trace_t *)VMA(1)`.
    results: *mut trace_t,
    /// Start point decoded by Raven as `(const float *)VMA(2)`.
    start: *const vec3_t,
    /// End point decoded by Raven as `(const float *)VMA(3)`.
    end: *const vec3_t,
    /// Trace mins decoded by Raven as `(const float *)VMA(4)`.
    mins: *const vec3_t,
    /// Trace maxs decoded by Raven as `(const float *)VMA(5)`.
    maxs: *const vec3_t,
    /// `clipHandle_t` model handle, read by Raven as raw `args[6]`.
    model: c_int,
    /// Contents mask, read by Raven as raw `args[7]`.
    brushmask: c_int,
    /// Model origin decoded by Raven as `(const float *)VMA(8)`.
    origin: *const vec3_t,
    /// Model angles decoded by Raven as `(const float *)VMA(9)`.
    angles: *const vec3_t,
}

impl CgCmTransformedboxtraceArgs {
    /// Construct raw `trap_CM_TransformedBoxTrace` syscall args.
    ///
    /// # Safety
    /// `results` must point to a writable `trace_t` slot, and all `vec3_t`
    /// pointers must remain valid for reads for the duration of the syscall.
    pub const unsafe fn new(
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

/// `CG_CM_TRANSFORMEDBOXTRACE` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_CM_TRANSFORMEDBOXTRACE, results, start, end,
/// mins, maxs, model, brushmask, origin, angles );`
/// Raven transport: `CM_TransformedBoxTrace(..., args[6], args[7], ...,
/// qfalse); return 0;`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:92`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:163-167`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:799-801`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:799-801`
pub struct CgCmTransformedboxtrace;

impl OutboundSysCall for CgCmTransformedboxtrace {
    type Import = MpCgameImport;
    type Args = CgCmTransformedboxtraceArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_TRANSFORMEDBOXTRACE;
}

impl EncodeSysCall for CgCmTransformedboxtrace {
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

impl DecodeSysCallReturn for CgCmTransformedboxtrace {
    // `CM_TransformedBoxTrace` writes through `results`; Raven returns 0 from the syscall arm.
    fn decode_return(_word: isize) -> Self::Output {}
}
