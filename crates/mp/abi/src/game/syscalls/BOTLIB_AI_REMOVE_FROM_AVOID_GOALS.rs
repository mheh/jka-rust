use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_REMOVE_FROM_AVOID_GOALS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiRemoveFromAvoidGoalsArgs {
    goalstate: c_int,
    number: c_int,
}

impl BotlibAiRemoveFromAvoidGoalsArgs {
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

/// `BOTLIB_AI_REMOVE_FROM_AVOID_GOALS` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:489`
pub struct BotlibAiRemoveFromAvoidGoals;

impl OutboundSysCall for BotlibAiRemoveFromAvoidGoals {
    type Import = MpGameImport;
    type Args = BotlibAiRemoveFromAvoidGoalsArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_REMOVE_FROM_AVOID_GOALS;
}

impl EncodeSysCall for BotlibAiRemoveFromAvoidGoals {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize, a.number as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiRemoveFromAvoidGoals {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
