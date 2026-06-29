use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;
use core::ffi::c_int;

/// `BOTLIB_AAS_AREA_REACHABILITY` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasAreaReachabilityArgs {
    areanum: c_int,
}

impl BotlibAasAreaReachabilityArgs {
    pub fn new(areanum: c_int) -> Self {
        Self { areanum }
    }

    pub fn areanum(&self) -> c_int {
        self.areanum
    }
}

/// `BOTLIB_AAS_AREA_REACHABILITY` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:375`
pub struct BotlibAasAreaReachability;

impl OutboundSysCall for BotlibAasAreaReachability {
    type Import = GameImport;
    type Args = BotlibAasAreaReachabilityArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_AREA_REACHABILITY;
}

impl EncodeSysCall for BotlibAasAreaReachability {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.areanum as isize])
    }
}

impl DecodeSysCallReturn for BotlibAasAreaReachability {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
