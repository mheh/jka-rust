use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AAS_FLOAT_FOR_BSP_EPAIR_KEY` outbound game-to-engine syscall.
///
/// C signature: `int trap_AAS_FloatForBSPEpairKey(int ent, char *key, float *value)`
#[derive(Debug)]
pub struct BotlibAasFloatForBspEpairKeyArgs {
    ent: c_int,
    key: CString,
    value: *mut f32,
}

impl BotlibAasFloatForBspEpairKeyArgs {
    pub fn new(ent: c_int, key: CString, value: *mut f32) -> Self {
        Self { ent, key, value }
    }

    pub fn ent(&self) -> c_int {
        self.ent
    }

    pub fn key(&self) -> &CString {
        &self.key
    }

    pub fn value(&self) -> *mut f32 {
        self.value
    }
}

/// `BOTLIB_AAS_FLOAT_FOR_BSP_EPAIR_KEY` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:372`
pub struct BotlibAasFloatForBspEpairKey;

impl OutboundSysCall for BotlibAasFloatForBspEpairKey {
    type Import = MpGameImport;
    type Args = BotlibAasFloatForBspEpairKeyArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AAS_FLOAT_FOR_BSP_EPAIR_KEY;
}

impl EncodeSysCall for BotlibAasFloatForBspEpairKey {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.ent as isize,
            ptr_to_word(a.key.as_ptr()),
            ptr_to_word(a.value),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasFloatForBspEpairKey {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
