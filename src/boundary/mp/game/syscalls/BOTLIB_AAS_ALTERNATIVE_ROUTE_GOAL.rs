use core::ffi::{c_int, c_void};

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AAS_ALTERNATIVE_ROUTE_GOAL` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasAlternativeRouteGoalArgs {
    start: *const vec3_t,
    startareanum: c_int,
    goal: *const vec3_t,
    goalareanum: c_int,
    travelflags: c_int,
    altroutegoals: *mut c_void,
    maxaltroutegoals: c_int,
    route_type: c_int,
}

impl BotlibAasAlternativeRouteGoalArgs {
    pub fn new(
        start: *const vec3_t,
        startareanum: c_int,
        goal: *const vec3_t,
        goalareanum: c_int,
        travelflags: c_int,
        altroutegoals: *mut c_void,
        maxaltroutegoals: c_int,
        route_type: c_int,
    ) -> Self {
        Self {
            start,
            startareanum,
            goal,
            goalareanum,
            travelflags,
            altroutegoals,
            maxaltroutegoals,
            route_type,
        }
    }

    pub fn start(&self) -> *const vec3_t { self.start }
    pub fn startareanum(&self) -> c_int { self.startareanum }
    pub fn goal(&self) -> *const vec3_t { self.goal }
    pub fn goalareanum(&self) -> c_int { self.goalareanum }
    pub fn travelflags(&self) -> c_int { self.travelflags }
    pub fn altroutegoals(&self) -> *mut c_void { self.altroutegoals }
    pub fn maxaltroutegoals(&self) -> c_int { self.maxaltroutegoals }
    pub fn route_type(&self) -> c_int { self.route_type }
}

pub struct BotlibAasAlternativeRouteGoal;

impl OutboundSysCall for BotlibAasAlternativeRouteGoal {
    type Import = GameImport;
    type Args = BotlibAasAlternativeRouteGoalArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_ALTERNATIVE_ROUTE_GOAL;
}

impl EncodeSysCall for BotlibAasAlternativeRouteGoal {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.start as *const u8),
            a.startareanum as isize,
            ptr_to_word(a.goal as *const u8),
            a.goalareanum as isize,
            a.travelflags as isize,
            ptr_to_word(a.altroutegoals as *const u8),
            a.maxaltroutegoals as isize,
            a.route_type as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasAlternativeRouteGoal {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
