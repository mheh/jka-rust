use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_ALLOC_CHAT_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiAllocChatStateArgs;

impl BotlibAiAllocChatStateArgs {
    pub fn new() -> Self {
        Self
    }
}

/// `BOTLIB_AI_ALLOC_CHAT_STATE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:421`
pub struct BotlibAiAllocChatState;

impl OutboundSysCall for BotlibAiAllocChatState {
    type Import = MpGameImport;
    type Args = BotlibAiAllocChatStateArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_ALLOC_CHAT_STATE;
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
