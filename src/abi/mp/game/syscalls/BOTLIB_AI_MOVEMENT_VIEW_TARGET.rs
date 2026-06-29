use core::ffi::{c_int, c_void};

use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_MOVEMENT_VIEW_TARGET` outbound game-to-engine syscall.
///
/// C signature:
/// ```c
/// int trap_BotMovementViewTarget(int movestate, void *goal, int travelflags,
///                                float lookahead, vec3_t target);
/// syscall(BOTLIB_AI_MOVEMENT_VIEW_TARGET, movestate, goal, travelflags,
///         PASSFLOAT(lookahead), target);
/// ```
#[derive(Debug)]
pub struct BotlibAiMovementViewTargetArgs {
    /// Bot move state handle.
    pub movestate: c_int,
    /// Pointer to `bot_goal_s` goal struct (opaque to the game module).
    pub goal: *mut c_void,
    /// Travel flags bitmask.
    pub travelflags: c_int,
    /// Look-ahead distance.
    pub lookahead: f32,
    /// Output: engine writes the computed view-target position here (vec3_t).
    pub target: *mut [f32; 3],
}

impl BotlibAiMovementViewTargetArgs {
    pub fn new(
        movestate: c_int,
        goal: *mut c_void,
        travelflags: c_int,
        lookahead: f32,
        target: *mut [f32; 3],
    ) -> Self {
        Self {
            movestate,
            goal,
            travelflags,
            lookahead,
            target,
        }
    }

    pub fn movestate(&self) -> c_int {
        self.movestate
    }
    pub fn goal(&self) -> *mut c_void {
        self.goal
    }
    pub fn travelflags(&self) -> c_int {
        self.travelflags
    }
    pub fn lookahead(&self) -> f32 {
        self.lookahead
    }
    pub fn target(&self) -> *mut [f32; 3] {
        self.target
    }
}

/// `BOTLIB_AI_MOVEMENT_VIEW_TARGET` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:470`
pub struct BotlibAiMovementViewTarget;

impl OutboundSysCall for BotlibAiMovementViewTarget {
    type Import = GameImport;
    type Args = BotlibAiMovementViewTargetArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_MOVEMENT_VIEW_TARGET;
}

impl EncodeSysCall for BotlibAiMovementViewTarget {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.movestate as isize,
            ptr_to_word(a.goal),
            a.travelflags as isize,
            pass_float(a.lookahead),
            ptr_to_word(a.target),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiMovementViewTarget {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
