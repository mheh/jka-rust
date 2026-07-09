use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_PC_LOAD_SOURCE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibPcLoadSourceArgs {
    filename: CString,
}

impl BotlibPcLoadSourceArgs {
    pub fn new(filename: CString) -> Self {
        Self { filename }
    }

    pub fn filename(&self) -> &CString {
        &self.filename
    }
}

/// `BOTLIB_PC_LOAD_SOURCE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:498`
pub struct BotlibPcLoadSource;

impl OutboundSysCall for BotlibPcLoadSource {
    type Import = MpGameImport;
    type Args = BotlibPcLoadSourceArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_PC_LOAD_SOURCE;
}

impl EncodeSysCall for BotlibPcLoadSource {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.filename.as_ptr())])
    }
}

impl DecodeSysCallReturn for BotlibPcLoadSource {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
