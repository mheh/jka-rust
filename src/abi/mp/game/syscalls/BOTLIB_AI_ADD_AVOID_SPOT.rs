use core::ffi::c_int;

use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport;
use crate::shared::vec3_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_ADD_AVOID_SPOT` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotAddAvoidSpot(int movestate, vec3_t origin, float radius, int type)`
#[derive(Debug)]
pub struct BotlibAiAddAvoidSpotArgs {
    movestate: c_int,
    origin: *const vec3_t,
    radius: f32,
    spot_type: c_int,
}

impl BotlibAiAddAvoidSpotArgs {
    pub fn new(movestate: c_int, origin: *const vec3_t, radius: f32, spot_type: c_int) -> Self {
        Self {
            movestate,
            origin,
            radius,
            spot_type,
        }
    }

    pub fn movestate(&self) -> c_int {
        self.movestate
    }
    pub fn origin(&self) -> *const vec3_t {
        self.origin
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
    pub fn spot_type(&self) -> c_int {
        self.spot_type
    }
}

/// `BOTLIB_AI_ADD_AVOID_SPOT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:493`
pub struct BotlibAiAddAvoidSpot;

impl OutboundSysCall for BotlibAiAddAvoidSpot {
    type Import = GameImport;
    type Args = BotlibAiAddAvoidSpotArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_ADD_AVOID_SPOT;
}

impl EncodeSysCall for BotlibAiAddAvoidSpot {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.movestate as isize,
            ptr_to_word(a.origin),
            pass_float(a.radius),
            a.spot_type as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiAddAvoidSpot {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
