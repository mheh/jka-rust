use core::ffi::{c_int, c_void};

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_ITEM_GOAL_IN_VIS_BUT_NOT_VISIBLE` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotItemGoalInVisButNotVisible(int viewer, vec3_t eye, vec3_t viewangles, bot_goal_t *goal)`
/// syscall args in order: viewer, eye (vec3_t ptr), viewangles (vec3_t ptr), goal (bot_goal_t *)
#[derive(Debug)]
pub struct BotlibAiItemGoalInVisButNotVisibleArgs {
    viewer: c_int,
    eye: *const f32,
    viewangles: *const f32,
    goal: *mut c_void,
}

impl BotlibAiItemGoalInVisButNotVisibleArgs {
    pub fn new(viewer: c_int, eye: *const f32, viewangles: *const f32, goal: *mut c_void) -> Self {
        Self {
            viewer,
            eye,
            viewangles,
            goal,
        }
    }

    pub fn viewer(&self) -> c_int {
        self.viewer
    }

    pub fn eye(&self) -> *const f32 {
        self.eye
    }

    pub fn viewangles(&self) -> *const f32 {
        self.viewangles
    }

    pub fn goal(&self) -> *mut c_void {
        self.goal
    }
}

/// `BOTLIB_AI_ITEM_GOAL_IN_VIS_BUT_NOT_VISIBLE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:453`
pub struct BotlibAiItemGoalInVisButNotVisible;

impl OutboundSysCall for BotlibAiItemGoalInVisButNotVisible {
    type Import = MpGameImport;
    type Args = BotlibAiItemGoalInVisButNotVisibleArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_ITEM_GOAL_IN_VIS_BUT_NOT_VISIBLE;
}

impl EncodeSysCall for BotlibAiItemGoalInVisButNotVisible {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.viewer as isize,
            ptr_to_word(a.eye),
            ptr_to_word(a.viewangles),
            ptr_to_word(a.goal),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiItemGoalInVisButNotVisible {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
