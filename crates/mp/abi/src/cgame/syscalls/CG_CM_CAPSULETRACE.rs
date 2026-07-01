use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_CM_CAPSULETRACE`.
///
/// Raven wrapper: `void trap_CM_CapsuleTrace(trace_t *results, const vec3_t start,
/// const vec3_t end, const vec3_t mins, const vec3_t maxs, clipHandle_t model,
/// int brushmask)`. The client switch decodes the five pointer payload words
/// with `VMA`, forwards `model`/`brushmask` as `args[6]`/`args[7]`, and calls
/// `CM_BoxTrace(..., qtrue)` to select capsule collision.
///
/// `cg_local.h` declares the neighboring `trap_CM_BoxTrace` but does not declare
/// `trap_CM_CapsuleTrace` in this Raven snapshot; the implemented wrapper and
/// switch arm are therefore the binding ABI evidence.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:157-160`
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:796-797`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:796-798`
#[derive(Debug)]
pub struct CgCmCapsuletraceArgs {
    /// Out pointer for the collision trace, decoded by Raven as `(trace_t *)VMA(1)`.
    results: *mut trace_t,
    /// Trace start vector, decoded by Raven as `(const float *)VMA(2)`.
    start: *const vec3_t,
    /// Trace end vector, decoded by Raven as `(const float *)VMA(3)`.
    end: *const vec3_t,
    /// Minimum capsule bounds, decoded by Raven as `(const float *)VMA(4)`.
    mins: *const vec3_t,
    /// Maximum capsule bounds, decoded by Raven as `(const float *)VMA(5)`.
    maxs: *const vec3_t,
    /// `clipHandle_t` model, read by Raven as `args[6]`.
    model: c_int,
    /// Collision contents mask, read by Raven as `args[7]`.
    brushmask: c_int,
}

impl CgCmCapsuletraceArgs {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
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

/// `CG_CM_CAPSULETRACE` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_CM_CAPSULETRACE, results, start, end, mins, maxs, model, brushmask );`
/// Raven transport: `CM_BoxTrace((trace_t *)VMA(1), (const float *)VMA(2),
/// (const float *)VMA(3), (const float *)VMA(4), (const float *)VMA(5),
/// args[6], args[7], qtrue); return 0;`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:91`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:157-160`
/// Args source: `oracle/oracle/codemp/client/cl_cgame.cpp:796-797`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:157-160`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:796-798`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:796-798`
pub struct CgCmCapsuletrace;

impl OutboundSysCall for CgCmCapsuletrace {
    type Import = MpCgameImport;
    type Args = CgCmCapsuletraceArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_CAPSULETRACE;
}

impl EncodeSysCall for CgCmCapsuletrace {
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

impl DecodeSysCallReturn for CgCmCapsuletrace {
    // Raven returns 0; the collision result is written through `results`.
    fn decode_return(_word: isize) -> Self::Output {}
}
