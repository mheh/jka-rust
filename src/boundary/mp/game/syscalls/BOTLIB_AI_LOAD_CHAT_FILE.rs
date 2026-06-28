use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_LOAD_CHAT_FILE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiLoadChatFileArgs {
    chatstate: c_int,
    chatfile: CString,
    chatname: CString,
}

impl BotlibAiLoadChatFileArgs {
    pub fn new(chatstate: c_int, chatfile: CString, chatname: CString) -> Self {
        Self { chatstate, chatfile, chatname }
    }

    pub fn chatstate(&self) -> c_int { self.chatstate }
    pub fn chatfile(&self) -> &CString { &self.chatfile }
    pub fn chatname(&self) -> &CString { &self.chatname }
}

pub struct BotlibAiLoadChatFile;

impl OutboundSysCall for BotlibAiLoadChatFile {
    type Import = GameImport;
    type Args = BotlibAiLoadChatFileArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_LOAD_CHAT_FILE;
}

impl EncodeSysCall for BotlibAiLoadChatFile {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.chatstate as isize,
            ptr_to_word(a.chatfile.as_ptr()),
            ptr_to_word(a.chatname.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiLoadChatFile {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
