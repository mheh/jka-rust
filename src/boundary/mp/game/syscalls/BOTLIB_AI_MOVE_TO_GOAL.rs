use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_MOVE_TO_GOAL` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotMoveToGoal(void *result, int movestate, void *goal, int travelflags)`
#[derive(Debug)]
pub struct BotlibAiMoveToGoalArgs {
    /// Out-param: engine writes the move result through this pointer (`bot_moveresult_s *`).
    result: *mut c_void,
    movestate: c_int,
    /// Pointer to the bot goal (`bot_goal_s *`).
    goal: *const c_void,
    travelflags: c_int,
}

impl BotlibAiMoveToGoalArgs {
    pub fn new(
        result: *mut c_void,
        movestate: c_int,
        goal: *const c_void,
        travelflags: c_int,
    ) -> Self {
        Self {
            result,
            movestate,
            goal,
            travelflags,
        }
    }

    pub fn result(&self) -> *mut c_void {
        self.result
    }
    pub fn movestate(&self) -> c_int {
        self.movestate
    }
    pub fn goal(&self) -> *const c_void {
        self.goal
    }
    pub fn travelflags(&self) -> c_int {
        self.travelflags
    }
}

/// `BOTLIB_AI_MOVE_TO_GOAL` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:465`
pub struct BotlibAiMoveToGoal;

impl OutboundSysCall for BotlibAiMoveToGoal {
    type Import = GameImport;
    type Args = BotlibAiMoveToGoalArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_MOVE_TO_GOAL;
}

impl EncodeSysCall for BotlibAiMoveToGoal {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.result as *const _),
            a.movestate as isize,
            ptr_to_word(a.goal),
            a.travelflags as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiMoveToGoal {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
