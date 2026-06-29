use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::{clipHandle_t, trace_t, vec3_t};

/// Arguments for `CG_CM_TRANSFORMEDBOXTRACE`.
///
/// Raven wrapper: `syscall( CG_CM_TRANSFORMEDBOXTRACE, results, start, end, mins, maxs, model, brushmask, origin, angles )`
/// Raven transport: `CM_TransformedBoxTrace((trace_t *)VMA(1), ... args[6], args[7], (const float *)VMA(8), (const float *)VMA(9))`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:161-166`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:542-544`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmTransformedboxtraceArgs {
    results: *mut trace_t,
    start: *const vec3_t,
    end: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    model: clipHandle_t,
    brushmask: c_int,
    origin: *const vec3_t,
    angles: *const vec3_t,
}

impl CgCmTransformedboxtraceArgs {
    pub const unsafe fn new(
        results: *mut trace_t,
        start: *const vec3_t,
        end: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        model: clipHandle_t,
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

    pub const fn model(&self) -> clipHandle_t {
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

/// `CG_CM_TRANSFORMEDBOXTRACE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:88`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:161-166`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:542-544`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:542-544`
pub struct CgCmTransformedboxtrace;

impl OutboundSysCall for CgCmTransformedboxtrace {
    type Import = SpCgameImport;
    type Args = CgCmTransformedboxtraceArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_TRANSFORMEDBOXTRACE;
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
    fn decode_return(_word: isize) -> Self::Output {}
}
