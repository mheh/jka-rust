use core::ffi::c_int;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;
use crate::shared::vec3_t;

/// `BOTLIB_AI_MOVE_IN_DIRECTION` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotMoveInDirection(int movestate, vec3_t dir, float speed, int type)`
/// Mirrors: `syscall(BOTLIB_AI_MOVE_IN_DIRECTION, movestate, dir, PASSFLOAT(speed), type)`
#[derive(Debug)]
pub struct BotlibAiMoveInDirectionArgs {
    movestate: c_int,
    dir: *const vec3_t,
    speed: f32,
    type_: c_int,
}

impl BotlibAiMoveInDirectionArgs {
    pub fn new(movestate: c_int, dir: *const vec3_t, speed: f32, type_: c_int) -> Self {
        Self {
            movestate,
            dir,
            speed,
            type_,
        }
    }

    pub fn movestate(&self) -> c_int {
        self.movestate
    }
    pub fn dir(&self) -> *const vec3_t {
        self.dir
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn type_(&self) -> c_int {
        self.type_
    }
}

/// `BOTLIB_AI_MOVE_IN_DIRECTION` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:466`
pub struct BotlibAiMoveInDirection;

impl OutboundSysCall for BotlibAiMoveInDirection {
    type Import = GameImport;
    type Args = BotlibAiMoveInDirectionArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_MOVE_IN_DIRECTION;
}

impl EncodeSysCall for BotlibAiMoveInDirection {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.movestate as isize,
            ptr_to_word(a.dir as *const u8),
            crate::ffi::syscalls::pass_float(a.speed),
            a.type_ as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiMoveInDirection {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
