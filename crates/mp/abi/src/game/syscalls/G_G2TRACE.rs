use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::vec3_t;

/// `G_G2TRACE` outbound game-to-engine syscall.
///
/// Maps to `trap_G2Trace` in `g_syscalls.c`:
/// ```c
/// void trap_G2Trace(trace_t *results, const vec3_t start, const vec3_t mins,
///                   const vec3_t maxs, const vec3_t end,
///                   int passEntityNum, int contentmask,
///                   int g2TraceType, int traceLod);
/// ```
#[derive(Debug)]
pub struct GG2TraceArgs {
    results: *mut trace_t,
    start: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    end: *const vec3_t,
    pass_entity_num: c_int,
    contentmask: c_int,
    g2_trace_type: c_int,
    trace_lod: c_int,
}

impl GG2TraceArgs {
    pub fn new(
        results: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        pass_entity_num: c_int,
        contentmask: c_int,
        g2_trace_type: c_int,
        trace_lod: c_int,
    ) -> Self {
        Self {
            results,
            start,
            mins,
            maxs,
            end,
            pass_entity_num,
            contentmask,
            g2_trace_type,
            trace_lod,
        }
    }

    pub fn results(&self) -> *mut trace_t {
        self.results
    }
    pub fn start(&self) -> *const vec3_t {
        self.start
    }
    pub fn mins(&self) -> *const vec3_t {
        self.mins
    }
    pub fn maxs(&self) -> *const vec3_t {
        self.maxs
    }
    pub fn end(&self) -> *const vec3_t {
        self.end
    }
    pub fn pass_entity_num(&self) -> c_int {
        self.pass_entity_num
    }
    pub fn contentmask(&self) -> c_int {
        self.contentmask
    }
    pub fn g2_trace_type(&self) -> c_int {
        self.g2_trace_type
    }
    pub fn trace_lod(&self) -> c_int {
        self.trace_lod
    }
}

/// `G_G2TRACE` MP game imports syscall ABI token.
///
/// Raven: ( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask );
/// Raven: collision detection against all linked entities with ghoul2 check
/// Source: `oracle/oracle/codemp/game/g_public.h:185`
pub struct GG2Trace;

impl OutboundSysCall for GG2Trace {
    type Import = MpGameImport;
    type Args = GG2TraceArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_G2TRACE;
}

impl EncodeSysCall for GG2Trace {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.results as *const _),
            ptr_to_word(a.start as *const _),
            ptr_to_word(a.mins as *const _),
            ptr_to_word(a.maxs as *const _),
            ptr_to_word(a.end as *const _),
            a.pass_entity_num as isize,
            a.contentmask as isize,
            a.g2_trace_type as isize,
            a.trace_lod as isize,
        ])
    }
}

impl DecodeSysCallReturn for GG2Trace {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
