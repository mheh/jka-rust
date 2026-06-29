use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_TOUCHING_GOAL` outbound game-to-engine syscall.
///
/// C: `int trap_BotTouchingGoal(vec3_t origin, void *goal)`
/// ABI: `syscall(BOTLIB_AI_TOUCHING_GOAL, origin, goal)`
#[derive(Debug)]
pub struct BotlibAiTouchingGoalArgs {
    /// Pointer to the bot's current origin (vec3_t — float[3]).
    origin: *const f32,
    /// Pointer to the bot_goal_s goal struct (opaque to the game module).
    goal: *mut c_void,
}

impl BotlibAiTouchingGoalArgs {
    pub fn new(origin: *const f32, goal: *mut c_void) -> Self {
        Self { origin, goal }
    }

    pub fn origin(&self) -> *const f32 {
        self.origin
    }

    pub fn goal(&self) -> *mut c_void {
        self.goal
    }
}

/// `BOTLIB_AI_TOUCHING_GOAL` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:452`
pub struct BotlibAiTouchingGoal;

impl OutboundSysCall for BotlibAiTouchingGoal {
    type Import = GameImport;
    type Args = BotlibAiTouchingGoalArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_TOUCHING_GOAL;
}

impl EncodeSysCall for BotlibAiTouchingGoal {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.origin), ptr_to_word(a.goal)])
    }
}

impl DecodeSysCallReturn for BotlibAiTouchingGoal {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
