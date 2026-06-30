use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_CHAT_LENGTH` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotChatLength(int chatstate)`
#[derive(Debug)]
pub struct BotlibAiChatLengthArgs {
    chatstate: c_int,
}

impl BotlibAiChatLengthArgs {
    pub fn new(chatstate: c_int) -> Self {
        Self { chatstate }
    }

    pub fn chatstate(&self) -> c_int {
        self.chatstate
    }
}

/// `BOTLIB_AI_CHAT_LENGTH` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:429`
pub struct BotlibAiChatLength;

impl OutboundSysCall for BotlibAiChatLength {
    type Import = MpGameImport;
    type Args = BotlibAiChatLengthArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_CHAT_LENGTH;
}

impl EncodeSysCall for BotlibAiChatLength {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.chatstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiChatLength {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
