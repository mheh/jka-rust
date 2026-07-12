use core::ffi::c_int;

use super::super::MpGameImport;
use mp_qshared::shared::vec3_t;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

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

/// `BOTLIB_AI_REACHABILITY_AREA` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:469`
pub struct BotlibAiReachabilityArea;

impl OutboundSysCall for BotlibAiReachabilityArea {
    type Import = MpGameImport;
    type Args = BotlibAiReachabilityAreaArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_REACHABILITY_AREA;
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
