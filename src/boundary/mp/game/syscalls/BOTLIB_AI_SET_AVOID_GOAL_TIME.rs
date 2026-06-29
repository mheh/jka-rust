use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;
use core::ffi::c_int;

/// `BOTLIB_AI_SET_AVOID_GOAL_TIME` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotSetAvoidGoalTime(int goalstate, int number, float avoidtime)`
/// Mirrors: `syscall(BOTLIB_AI_SET_AVOID_GOAL_TIME, goalstate, number, PASSFLOAT(avoidtime))`
#[derive(Debug)]
pub struct BotlibAiSetAvoidGoalTimeArgs {
    goalstate: c_int,
    number: c_int,
    avoidtime: f32,
}

impl BotlibAiSetAvoidGoalTimeArgs {
    pub fn new(goalstate: c_int, number: c_int, avoidtime: f32) -> Self {
        Self {
            goalstate,
            number,
            avoidtime,
        }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }
    pub fn number(&self) -> c_int {
        self.number
    }
    pub fn avoidtime(&self) -> f32 {
        self.avoidtime
    }
}

/// `BOTLIB_AI_SET_AVOID_GOAL_TIME` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:492`
pub struct BotlibAiSetAvoidGoalTime;

impl OutboundSysCall for BotlibAiSetAvoidGoalTime {
    type Import = GameImport;
    type Args = BotlibAiSetAvoidGoalTimeArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_SET_AVOID_GOAL_TIME;
}

impl EncodeSysCall for BotlibAiSetAvoidGoalTime {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.goalstate as isize,
            a.number as isize,
            crate::ffi::syscalls::pass_float(a.avoidtime),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiSetAvoidGoalTime {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
