use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_STRING_CONTAINS` outbound game-to-engine syscall.
///
/// C ABI: `int trap_StringContains(char *str1, char *str2, int casesensitive)`
#[derive(Debug)]
pub struct BotlibAiStringContainsArgs {
    str1: CString,
    str2: CString,
    casesensitive: c_int,
}

impl BotlibAiStringContainsArgs {
    pub fn new(str1: CString, str2: CString, casesensitive: c_int) -> Self {
        Self {
            str1,
            str2,
            casesensitive,
        }
    }

    pub fn str1(&self) -> &CString {
        &self.str1
    }
    pub fn str2(&self) -> &CString {
        &self.str2
    }
    pub fn casesensitive(&self) -> c_int {
        self.casesensitive
    }
}

/// `BOTLIB_AI_STRING_CONTAINS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:431`
pub struct BotlibAiStringContains;

impl OutboundSysCall for BotlibAiStringContains {
    type Import = MpGameImport;
    type Args = BotlibAiStringContainsArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_STRING_CONTAINS;
}

impl EncodeSysCall for BotlibAiStringContains {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.str1.as_ptr()),
            ptr_to_word(a.str2.as_ptr()),
            a.casesensitive as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiStringContains {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
