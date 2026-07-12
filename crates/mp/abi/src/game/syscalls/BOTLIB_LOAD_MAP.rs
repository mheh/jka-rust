use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_LOAD_MAP` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(BOTLIB_LOAD_MAP, m.as_ptr())` from `trap_BotLibLoadMap`.
#[derive(Debug)]
pub struct BotlibLoadMapArgs {
    /// Null-terminated map name string.
    mapname: CString,
}

impl BotlibLoadMapArgs {
    pub fn new(mapname: CString) -> Self {
        Self { mapname }
    }

    pub fn mapname(&self) -> &CString {
        &self.mapname
    }
}

/// `BOTLIB_LOAD_MAP` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:348`
pub struct BotlibLoadMap;

impl OutboundSysCall for BotlibLoadMap {
    type Import = MpGameImport;
    type Args = BotlibLoadMapArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_LOAD_MAP;
}

impl EncodeSysCall for BotlibLoadMap {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.mapname.as_ptr())])
    }
}

impl DecodeSysCallReturn for BotlibLoadMap {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
