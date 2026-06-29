use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_SET_CHAT_NAME` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotSetChatName(int chatstate, char *name, int client)`
#[derive(Debug)]
pub struct BotlibAiSetChatNameArgs {
    chatstate: c_int,
    name: CString,
    client: c_int,
}

impl BotlibAiSetChatNameArgs {
    pub fn new(chatstate: c_int, name: CString, client: c_int) -> Self {
        Self {
            chatstate,
            name,
            client,
        }
    }

    pub fn chatstate(&self) -> c_int {
        self.chatstate
    }
    pub fn name(&self) -> &CString {
        &self.name
    }
    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_AI_SET_CHAT_NAME` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:438`
pub struct BotlibAiSetChatName;

impl OutboundSysCall for BotlibAiSetChatName {
    type Import = GameImport;
    type Args = BotlibAiSetChatNameArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_SET_CHAT_NAME;
}

impl EncodeSysCall for BotlibAiSetChatName {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.chatstate as isize,
            ptr_to_word(a.name.as_ptr()),
            a.client as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiSetChatName {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
