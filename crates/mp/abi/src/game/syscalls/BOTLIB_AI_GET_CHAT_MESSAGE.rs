use core::ffi::{c_char, c_int};

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_GET_CHAT_MESSAGE` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotGetChatMessage(int chatstate, char *buf, int size)`
#[derive(Debug)]
pub struct BotlibAiGetChatMessageArgs {
    chatstate: c_int,
    buf: *mut c_char,
    size: c_int,
}

impl BotlibAiGetChatMessageArgs {
    pub fn new(chatstate: c_int, buf: *mut c_char, size: c_int) -> Self {
        Self {
            chatstate,
            buf,
            size,
        }
    }

    pub fn chatstate(&self) -> c_int {
        self.chatstate
    }

    pub fn buf(&self) -> *mut c_char {
        self.buf
    }

    pub fn size(&self) -> c_int {
        self.size
    }
}

/// `BOTLIB_AI_GET_CHAT_MESSAGE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:488`
pub struct BotlibAiGetChatMessage;

impl OutboundSysCall for BotlibAiGetChatMessage {
    type Import = MpGameImport;
    type Args = BotlibAiGetChatMessageArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_GET_CHAT_MESSAGE;
}

impl EncodeSysCall for BotlibAiGetChatMessage {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.chatstate as isize, ptr_to_word(a.buf), a.size as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiGetChatMessage {
    fn decode_return(_word: isize) -> Self::Output {}
}
