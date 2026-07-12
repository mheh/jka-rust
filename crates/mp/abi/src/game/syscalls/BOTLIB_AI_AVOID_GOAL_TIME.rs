use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_AVOID_GOAL_TIME` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiAvoidGoalTimeArgs {
    goalstate: c_int,
    number: c_int,
}

impl BotlibAiAvoidGoalTimeArgs {
    pub fn new(goalstate: c_int, number: c_int) -> Self {
        Self { goalstate, number }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }

    pub fn number(&self) -> c_int {
        self.number
    }
}

/// `BOTLIB_AI_AVOID_GOAL_TIME` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:455`
pub struct BotlibAiAvoidGoalTime;

impl OutboundSysCall for BotlibAiAvoidGoalTime {
    type Import = MpGameImport;
    type Args = BotlibAiAvoidGoalTimeArgs;
    type Output = f32;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_AVOID_GOAL_TIME;
}

impl EncodeSysCall for BotlibAiAvoidGoalTime {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize, a.number as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiAvoidGoalTime {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
