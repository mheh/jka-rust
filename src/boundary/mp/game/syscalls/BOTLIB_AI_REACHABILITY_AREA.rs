use core::ffi::c_int;

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_REACHABILITY_AREA` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiReachabilityAreaArgs {
    origin: *const vec3_t,
    testground: c_int,
}

impl BotlibAiReachabilityAreaArgs {
    pub fn new(origin: *const vec3_t, testground: c_int) -> Self {
        Self { origin, testground }
    }

    pub fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub fn testground(&self) -> c_int {
        self.testground
    }
}

pub struct BotlibAiReachabilityArea;

impl OutboundSysCall for BotlibAiReachabilityArea {
    type Import = GameImport;
    type Args = BotlibAiReachabilityAreaArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_REACHABILITY_AREA;
}

impl EncodeSysCall for BotlibAiReachabilityArea {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.origin), a.testground as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiReachabilityArea {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
