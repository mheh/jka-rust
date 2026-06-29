use core::ffi::c_int;
use std::ffi::CString;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

/// `BOTLIB_AI_QUEUE_CONSOLE_MESSAGE` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotQueueConsoleMessage(int chatstate, int type, char *message)`
#[derive(Debug)]
pub struct BotlibAiQueueConsoleMessageArgs {
    chatstate: c_int,
    r#type: c_int,
    message: CString,
}

impl BotlibAiQueueConsoleMessageArgs {
    pub fn new(chatstate: c_int, r#type: c_int, message: CString) -> Self {
        Self {
            chatstate,
            r#type,
            message,
        }
    }

    pub fn chatstate(&self) -> c_int {
        self.chatstate
    }
    pub fn r#type(&self) -> c_int {
        self.r#type
    }
    pub fn message(&self) -> &CString {
        &self.message
    }
}

/// `BOTLIB_AI_QUEUE_CONSOLE_MESSAGE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:423`
pub struct BotlibAiQueueConsoleMessage;

impl OutboundSysCall for BotlibAiQueueConsoleMessage {
    type Import = GameImport;
    type Args = BotlibAiQueueConsoleMessageArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_QUEUE_CONSOLE_MESSAGE;
}

impl EncodeSysCall for BotlibAiQueueConsoleMessage {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.chatstate as isize,
            a.r#type as isize,
            ptr_to_word(a.message.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiQueueConsoleMessage {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
