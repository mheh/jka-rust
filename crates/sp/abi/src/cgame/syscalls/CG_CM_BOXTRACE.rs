use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::common::sp::trace_t::trace_t;
use sp_qshared::shared::clipHandle_t;
use sp_qshared::shared::vec3_t;

/// Arguments for `CG_CM_BOXTRACE`.
///
/// Raven wrapper: `syscall( CG_CM_BOXTRACE, results, start, end, mins, maxs, model, brushmask )`
/// Raven transport: `CM_BoxTrace((trace_t *)VMA(1), ... args[6], args[7])`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:155-159`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:539-541`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmBoxtraceArgs {
    results: *mut trace_t,
    start: *const vec3_t,
    end: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    model: clipHandle_t,
    brushmask: c_int,
}

impl CgCmBoxtraceArgs {
    pub const unsafe fn new(
        results: *mut trace_t,
        start: *const vec3_t,
        end: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        model: clipHandle_t,
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

    pub const fn model(&self) -> clipHandle_t {
        self.model
    }

    pub const fn brushmask(&self) -> c_int {
        self.brushmask
    }
}

/// `CG_CM_BOXTRACE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:87`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:155-159`
/// Output source: `oracle/code/client/cl_cgame.cpp:539-541`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:539-541`
pub struct CgCmBoxtrace;

impl OutboundSysCall for CgCmBoxtrace {
    type Import = SpCgameImport;
    type Args = CgCmBoxtraceArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_BOXTRACE;
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
    fn decode_return(_word: isize) -> Self::Output {}
}
