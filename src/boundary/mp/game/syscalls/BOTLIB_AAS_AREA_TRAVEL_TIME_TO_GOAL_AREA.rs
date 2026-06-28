use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::codemp::game::q_shared_h::vec3_t;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AAS_AREA_TRAVEL_TIME_TO_GOAL_AREA` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasAreaTravelTimeToGoalAreaArgs {
    areanum: c_int,
    origin: *const vec3_t,
    goalareanum: c_int,
    travelflags: c_int,
}

impl BotlibAasAreaTravelTimeToGoalAreaArgs {
    pub fn new(areanum: c_int, origin: *const vec3_t, goalareanum: c_int, travelflags: c_int) -> Self {
        Self { areanum, origin, goalareanum, travelflags }
    }

    pub fn areanum(&self) -> c_int { self.areanum }
    pub fn origin(&self) -> *const vec3_t { self.origin }
    pub fn goalareanum(&self) -> c_int { self.goalareanum }
    pub fn travelflags(&self) -> c_int { self.travelflags }
}

pub struct BotlibAasAreaTravelTimeToGoalArea;

impl OutboundSysCall for BotlibAasAreaTravelTimeToGoalArea {
    type Import = GameImport;
    type Args = BotlibAasAreaTravelTimeToGoalAreaArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_AREA_TRAVEL_TIME_TO_GOAL_AREA;
}

impl EncodeSysCall for BotlibAasAreaTravelTimeToGoalArea {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.areanum as isize,
            ptr_to_word(a.origin as *const u8),
            a.goalareanum as isize,
            a.travelflags as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasAreaTravelTimeToGoalArea {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
