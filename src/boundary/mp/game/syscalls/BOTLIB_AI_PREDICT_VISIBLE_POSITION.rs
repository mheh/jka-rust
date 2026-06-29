use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_PREDICT_VISIBLE_POSITION` outbound game-to-engine syscall.
///
/// C signature:
/// ```c
/// int trap_BotPredictVisiblePosition(vec3_t origin, int areanum,
///                                    void *goal, int travelflags,
///                                    vec3_t target);
/// syscall(BOTLIB_AI_PREDICT_VISIBLE_POSITION, origin, areanum, goal, travelflags, target);
/// ```
#[derive(Debug)]
pub struct BotlibAiPredictVisiblePositionArgs {
    /// Starting origin position (vec3_t pointer).
    pub origin: *const [f32; 3],
    /// AAS area number for the origin.
    pub areanum: c_int,
    /// Pointer to `bot_goal_s` goal struct (opaque to the game module).
    pub goal: *mut c_void,
    /// Travel flags bitmask.
    pub travelflags: c_int,
    /// Output: engine writes the predicted visible position here (vec3_t).
    pub target: *mut [f32; 3],
}

impl BotlibAiPredictVisiblePositionArgs {
    pub fn new(
        origin: *const [f32; 3],
        areanum: c_int,
        goal: *mut c_void,
        travelflags: c_int,
        target: *mut [f32; 3],
    ) -> Self {
        Self {
            origin,
            areanum,
            goal,
            travelflags,
            target,
        }
    }

    pub fn origin(&self) -> *const [f32; 3] {
        self.origin
    }
    pub fn areanum(&self) -> c_int {
        self.areanum
    }
    pub fn goal(&self) -> *mut c_void {
        self.goal
    }
    pub fn travelflags(&self) -> c_int {
        self.travelflags
    }
    pub fn target(&self) -> *mut [f32; 3] {
        self.target
    }
}

/// `BOTLIB_AI_PREDICT_VISIBLE_POSITION` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:490`
pub struct BotlibAiPredictVisiblePosition;

impl OutboundSysCall for BotlibAiPredictVisiblePosition {
    type Import = GameImport;
    type Args = BotlibAiPredictVisiblePositionArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_PREDICT_VISIBLE_POSITION;
}

impl EncodeSysCall for BotlibAiPredictVisiblePosition {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.origin),
            a.areanum as isize,
            ptr_to_word(a.goal),
            a.travelflags as isize,
            ptr_to_word(a.target),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiPredictVisiblePosition {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
