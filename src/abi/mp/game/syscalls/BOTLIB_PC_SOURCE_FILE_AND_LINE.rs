use core::ffi::{c_char, c_int};

use super::super::MpGameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_PC_SOURCE_FILE_AND_LINE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibPcSourceFileAndLineArgs {
    handle: c_int,
    filename: *mut c_char,
    line: *mut c_int,
}

impl BotlibPcSourceFileAndLineArgs {
    pub fn new(handle: c_int, filename: *mut c_char, line: *mut c_int) -> Self {
        Self {
            handle,
            filename,
            line,
        }
    }

    pub fn handle(&self) -> c_int {
        self.handle
    }
    pub fn filename(&self) -> *mut c_char {
        self.filename
    }
    pub fn line(&self) -> *mut c_int {
        self.line
    }
}

/// `BOTLIB_PC_SOURCE_FILE_AND_LINE` MP game imports syscall ABI token.
///
/// Raven: Ghoul2 Insert Start
/// Source: `oracle/oracle/codemp/game/g_public.h:501`
pub struct BotlibPcSourceFileAndLine;

impl OutboundSysCall for BotlibPcSourceFileAndLine {
    type Import = MpGameImport;
    type Args = BotlibPcSourceFileAndLineArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_PC_SOURCE_FILE_AND_LINE;
}

impl EncodeSysCall for BotlibPcSourceFileAndLine {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.handle as isize,
            ptr_to_word(a.filename),
            ptr_to_word(a.line),
        ])
    }
}

impl DecodeSysCallReturn for BotlibPcSourceFileAndLine {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
