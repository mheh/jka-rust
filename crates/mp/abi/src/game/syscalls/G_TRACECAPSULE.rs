use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::vec3_t;

/// `G_TRACECAPSULE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GTracecapsuleArgs {
    results: *mut trace_t,
    start: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    end: *const vec3_t,
    pass_entity_num: c_int,
    contentmask: c_int,
}

impl GTracecapsuleArgs {
    pub fn new(
        results: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        pass_entity_num: c_int,
        contentmask: c_int,
    ) -> Self {
        Self {
            results,
            start,
            mins,
            maxs,
            end,
            pass_entity_num,
            contentmask,
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
}

/// `G_TRACECAPSULE` MP game imports syscall ABI token.
///
/// Raven: ( trace_t *results, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end, int passEntityNum, int contentmask );
/// Source: `oracle/codemp/game/g_public.h:235`
pub struct GTracecapsule;

impl OutboundSysCall for GTracecapsule {
    type Import = MpGameImport;
    type Args = GTracecapsuleArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_TRACECAPSULE;
}

impl EncodeSysCall for GTracecapsule {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.results as *const _),
            ptr_to_word(a.start as *const _),
            ptr_to_word(a.mins as *const _),
            ptr_to_word(a.maxs as *const _),
            ptr_to_word(a.end as *const _),
            a.pass_entity_num as isize,
            a.contentmask as isize,
            0,
            10,
        ])
    }
}

impl DecodeSysCallReturn for GTracecapsule {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
