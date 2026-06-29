use core::ffi::{c_char, c_int};

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AAS_VALUE_FOR_BSP_EPAIR_KEY` outbound game-to-engine syscall.
///
/// C ABI: `int trap_AAS_ValueForBSPEpairKey(int ent, char *key, char *value, int size)`
#[derive(Debug)]
pub struct BotlibAasValueForBspEpairKeyArgs {
    ent: c_int,
    key: *const c_char,
    value: *mut c_char,
    size: c_int,
}

impl BotlibAasValueForBspEpairKeyArgs {
    pub fn new(ent: c_int, key: *const c_char, value: *mut c_char, size: c_int) -> Self {
        Self {
            ent,
            key,
            value,
            size,
        }
    }

    pub fn ent(&self) -> c_int {
        self.ent
    }
    pub fn key(&self) -> *const c_char {
        self.key
    }
    pub fn value(&self) -> *mut c_char {
        self.value
    }
    pub fn size(&self) -> c_int {
        self.size
    }
}

/// `BOTLIB_AAS_VALUE_FOR_BSP_EPAIR_KEY` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:370`
pub struct BotlibAasValueForBspEpairKey;

impl OutboundSysCall for BotlibAasValueForBspEpairKey {
    type Import = GameImport;
    type Args = BotlibAasValueForBspEpairKeyArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AAS_VALUE_FOR_BSP_EPAIR_KEY;
}

impl EncodeSysCall for BotlibAasValueForBspEpairKey {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.ent as isize,
            ptr_to_word(a.key),
            ptr_to_word(a.value),
            a.size as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAasValueForBspEpairKey {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
