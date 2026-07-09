use core::ffi::{c_char, c_int};
use std::ffi::CString;

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_PC_ADD_GLOBAL_DEFINE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibPcAddGlobalDefineArgs {
    string: CString,
}

impl BotlibPcAddGlobalDefineArgs {
    pub fn new(string: CString) -> Self {
        Self { string }
    }

    pub fn string(&self) -> *const c_char {
        self.string.as_ptr()
    }
}

/// `BOTLIB_PC_ADD_GLOBAL_DEFINE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:346`
pub struct BotlibPcAddGlobalDefine;

impl OutboundSysCall for BotlibPcAddGlobalDefine {
    type Import = MpGameImport;
    type Args = BotlibPcAddGlobalDefineArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_PC_ADD_GLOBAL_DEFINE;
}

impl EncodeSysCall for BotlibPcAddGlobalDefine {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.string())])
    }
}

impl DecodeSysCallReturn for BotlibPcAddGlobalDefine {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
