use core::ffi::{c_char, c_int};
use std::ffi::CString;

use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_LIBVAR_GET` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotLibVarGet(char *var_name, char *value, int size)`
/// Syscall: `BOTLIB_LIBVAR_GET, var_name, value, size`
#[derive(Debug)]
pub struct BotlibLibvarGetArgs {
    /// Name of the libvar to query (input string).
    var_name: CString,
    /// Caller-allocated buffer the engine writes the value into (out-param).
    value: *mut c_char,
    /// Size of the `value` buffer in bytes.
    size: c_int,
}

impl BotlibLibvarGetArgs {
    pub fn new(var_name: CString, value: *mut c_char, size: c_int) -> Self {
        Self { var_name, value, size }
    }

    pub fn var_name(&self) -> &CString {
        &self.var_name
    }

    pub fn value(&self) -> *mut c_char {
        self.value
    }

    pub fn size(&self) -> c_int {
        self.size
    }
}

pub struct BotlibLibvarGet;

impl OutboundSysCall for BotlibLibvarGet {
    type Args = BotlibLibvarGetArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_LIBVAR_GET;
}

impl EncodeSysCall for BotlibLibvarGet {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.var_name.as_ptr()),
            ptr_to_word(a.value),
            a.size as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibLibvarGet {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
