use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::shared::vec3_t;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AAS_POINT_REACHABILITY_AREA_INDEX` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasPointReachabilityAreaIndexArgs {
    point: *const vec3_t,
}

impl BotlibAasPointReachabilityAreaIndexArgs {
    pub fn new(point: *const vec3_t) -> Self {
        Self { point }
    }

    pub fn point(&self) -> *const vec3_t {
        self.point
    }
}

/// `BOTLIB_AAS_POINT_REACHABILITY_AREA_INDEX` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:496`
pub struct BotlibAasPointReachabilityAreaIndex;

impl OutboundSysCall for BotlibAasPointReachabilityAreaIndex {
    type Import = GameImport;
    type Args = BotlibAasPointReachabilityAreaIndexArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_POINT_REACHABILITY_AREA_INDEX;
}

impl EncodeSysCall for BotlibAasPointReachabilityAreaIndex {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.point)])
    }
}

impl DecodeSysCallReturn for BotlibAasPointReachabilityAreaIndex {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
