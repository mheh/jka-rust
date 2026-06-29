use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AAS_INT_FOR_BSP_EPAIR_KEY` outbound game-to-engine syscall.
///
/// Mirrors: `int trap_AAS_IntForBSPEpairKey(int ent, char *key, int *value)`
/// C ABI:   `syscall(BOTLIB_AAS_INT_FOR_BSP_EPAIR_KEY, ent, key, value)`
#[derive(Debug)]
pub struct BotlibAasIntForBspEpairKeyArgs {
    pub ent: c_int,
    pub key: CString,
    pub value: *mut c_int,
}

impl BotlibAasIntForBspEpairKeyArgs {
    pub fn new(ent: c_int, key: CString, value: *mut c_int) -> Self {
        Self { ent, key, value }
    }

    pub fn ent(&self) -> c_int {
        self.ent
    }

    pub fn key(&self) -> &CString {
        &self.key
    }

    pub fn value(&self) -> *mut c_int {
        self.value
    }
}

/// `BOTLIB_AAS_INT_FOR_BSP_EPAIR_KEY` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:373`
pub struct BotlibAasIntForBspEpairKey;

impl OutboundSysCall for BotlibAasIntForBspEpairKey {
    type Import = GameImport;
    type Args = BotlibAasIntForBspEpairKeyArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_INT_FOR_BSP_EPAIR_KEY;
}

impl EncodeSysCall for BotlibAasIntForBspEpairKey {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.ent as isize,
            ptr_to_word(a.key.as_ptr()),
            ptr_to_word(a.value),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasIntForBspEpairKey {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
