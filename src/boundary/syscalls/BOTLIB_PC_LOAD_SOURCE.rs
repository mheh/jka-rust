use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct BotlibPcLoadSource;

impl OutboundSysCall for BotlibPcLoadSource {
    type Args = BotlibPcLoadSourceArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_PC_LOAD_SOURCE;
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
