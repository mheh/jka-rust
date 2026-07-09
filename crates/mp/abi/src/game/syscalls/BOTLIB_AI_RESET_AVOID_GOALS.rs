use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_RESET_AVOID_GOALS` outbound game-to-engine syscall.
///
/// Mirrors `trap_BotResetAvoidGoals(goalstate: i32)` → void.
#[derive(Debug)]
pub struct BotlibAiResetAvoidGoalsArgs {
    /// Bot goal-state handle.
    goalstate: c_int,
}

impl BotlibAiResetAvoidGoalsArgs {
    pub fn new(goalstate: c_int) -> Self {
        Self { goalstate }
    }

    pub fn goalstate(&self) -> c_int {
        self.goalstate
    }
}

/// `BOTLIB_AI_RESET_AVOID_GOALS` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:441`
pub struct BotlibAiResetAvoidGoals;

impl OutboundSysCall for BotlibAiResetAvoidGoals {
    type Import = MpGameImport;
    type Args = BotlibAiResetAvoidGoalsArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_RESET_AVOID_GOALS;
}

impl EncodeSysCall for BotlibAiResetAvoidGoals {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.goalstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiResetAvoidGoals {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
