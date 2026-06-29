use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_NUM_INITIAL_CHATS` outbound game-to-engine syscall.
///
/// C signature: `int trap_BotNumInitialChats(int chatstate, char *type)`
#[derive(Debug)]
pub struct BotlibAiNumInitialChatsArgs {
    pub chatstate: c_int,
    pub chat_type: CString,
}

impl BotlibAiNumInitialChatsArgs {
    pub fn new(chatstate: c_int, chat_type: CString) -> Self {
        Self {
            chatstate,
            chat_type,
        }
    }

    pub fn chatstate(&self) -> c_int {
        self.chatstate
    }

    pub fn chat_type(&self) -> &CString {
        &self.chat_type
    }
}

/// `BOTLIB_AI_NUM_INITIAL_CHATS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:487`
pub struct BotlibAiNumInitialChats;

impl OutboundSysCall for BotlibAiNumInitialChats {
    type Import = GameImport;
    type Args = BotlibAiNumInitialChatsArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_NUM_INITIAL_CHATS;
}

impl EncodeSysCall for BotlibAiNumInitialChats {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.chatstate as isize, ptr_to_word(a.chat_type.as_ptr())])
    }
}

impl DecodeSysCallReturn for BotlibAiNumInitialChats {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
