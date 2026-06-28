use core::ffi::c_int;

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AAS_TRACE_AREAS` outbound game-to-engine syscall.
///
/// C: `int trap_AAS_TraceAreas(vec3_t start, vec3_t end, int *areas, vec3_t *points, int maxareas)`
#[derive(Debug)]
pub struct BotlibAasTraceAreasArgs {
    /// Start point of the trace (vec3_t passed by pointer).
    pub start: *const vec3_t,
    /// End point of the trace (vec3_t passed by pointer).
    pub end: *const vec3_t,
    /// Output array of area numbers (engine writes through this pointer).
    pub areas: *mut c_int,
    /// Output array of intersection points (engine writes through this pointer).
    pub points: *mut vec3_t,
    /// Maximum number of areas to return.
    pub maxareas: c_int,
}

impl BotlibAasTraceAreasArgs {
    pub fn new(
        start: *const vec3_t,
        end: *const vec3_t,
        areas: *mut c_int,
        points: *mut vec3_t,
        maxareas: c_int,
    ) -> Self {
        Self { start, end, areas, points, maxareas }
    }

    pub fn start(&self) -> *const vec3_t { self.start }
    pub fn end(&self) -> *const vec3_t { self.end }
    pub fn areas(&self) -> *mut c_int { self.areas }
    pub fn points(&self) -> *mut vec3_t { self.points }
    pub fn maxareas(&self) -> c_int { self.maxareas }
}

pub struct BotlibAasTraceAreas;

impl OutboundSysCall for BotlibAasTraceAreas {
    type Args = BotlibAasTraceAreasArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_TRACE_AREAS;
}

impl EncodeSysCall for BotlibAasTraceAreas {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.start as *const u8),
            ptr_to_word(a.end as *const u8),
            ptr_to_word(a.areas as *const u8),
            ptr_to_word(a.points as *const u8),
            a.maxareas as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasTraceAreas {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
