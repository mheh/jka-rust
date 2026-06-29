use core::ffi::{c_int, c_void};

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

/// `BOTLIB_AAS_ENTITY_INFO` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAasEntityInfoArgs {
    entnum: c_int,
    info: *mut c_void,
}

impl BotlibAasEntityInfoArgs {
    pub fn new(entnum: c_int, info: *mut c_void) -> Self {
        Self { entnum, info }
    }

    pub fn entnum(&self) -> c_int {
        self.entnum
    }

    pub fn info(&self) -> *mut c_void {
        self.info
    }
}

/// `BOTLIB_AAS_ENTITY_INFO` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:359`
pub struct BotlibAasEntityInfo;

impl OutboundSysCall for BotlibAasEntityInfo {
    type Import = GameImport;
    type Args = BotlibAasEntityInfoArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_ENTITY_INFO;
}

impl EncodeSysCall for BotlibAasEntityInfo {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.entnum as isize, ptr_to_word(a.info)])
    }
}

impl DecodeSysCallReturn for BotlibAasEntityInfo {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
