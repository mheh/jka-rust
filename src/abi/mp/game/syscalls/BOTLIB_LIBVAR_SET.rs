use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_LIBVAR_SET` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibLibvarSetArgs {
    var_name: CString,
    value: CString,
}

impl BotlibLibvarSetArgs {
    pub fn new(var_name: CString, value: CString) -> Self {
        Self { var_name, value }
    }

    pub fn var_name(&self) -> &CString {
        &self.var_name
    }

    pub fn value(&self) -> &CString {
        &self.value
    }
}

/// `BOTLIB_LIBVAR_SET` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:344`
pub struct BotlibLibvarSet;

impl OutboundSysCall for BotlibLibvarSet {
    type Import = MpGameImport;
    type Args = BotlibLibvarSetArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_LIBVAR_SET;
}

impl EncodeSysCall for BotlibLibvarSet {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.var_name.as_ptr()),
            ptr_to_word(a.value.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for BotlibLibvarSet {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
