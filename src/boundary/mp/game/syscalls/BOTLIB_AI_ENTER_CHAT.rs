use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_ENTER_CHAT` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotEnterChat(int chatstate, int client, int sendto)`
#[derive(Debug)]
pub struct BotlibAiEnterChatArgs {
    chatstate: c_int,
    client: c_int,
    sendto: c_int,
}

impl BotlibAiEnterChatArgs {
    pub fn new(chatstate: c_int, client: c_int, sendto: c_int) -> Self {
        Self { chatstate, client, sendto }
    }

    pub fn chatstate(&self) -> c_int { self.chatstate }
    pub fn client(&self) -> c_int { self.client }
    pub fn sendto(&self) -> c_int { self.sendto }
}

pub struct BotlibAiEnterChat;

impl OutboundSysCall for BotlibAiEnterChat {
    type Import = GameImport;
    type Args = BotlibAiEnterChatArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_ENTER_CHAT;
}

impl EncodeSysCall for BotlibAiEnterChat {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.chatstate as isize,
            a.client as isize,
            a.sendto as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiEnterChat {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
