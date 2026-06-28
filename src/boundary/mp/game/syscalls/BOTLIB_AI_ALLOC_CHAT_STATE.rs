use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_ALLOC_CHAT_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiAllocChatStateArgs;

impl BotlibAiAllocChatStateArgs {
    pub fn new() -> Self {
        Self
    }
}

pub struct BotlibAiAllocChatState;

impl OutboundSysCall for BotlibAiAllocChatState {
    type Import = GameImport;
    type Args = BotlibAiAllocChatStateArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_ALLOC_CHAT_STATE;
}

impl EncodeSysCall for BotlibAiAllocChatState {
    fn encode_syscall(_a: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for BotlibAiAllocChatState {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
