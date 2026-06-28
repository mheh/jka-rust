use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct BotlibLoadMap;

impl OutboundSysCall for BotlibLoadMap {
    type Args = BotlibLoadMapArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_LOAD_MAP;
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
