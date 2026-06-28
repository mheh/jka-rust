use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_FREE_CHAT_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiFreeChatStateArgs {
    handle: c_int,
}

impl BotlibAiFreeChatStateArgs {
    pub fn new(handle: c_int) -> Self {
        Self { handle }
    }

    pub fn handle(&self) -> c_int {
        self.handle
    }
}

pub struct BotlibAiFreeChatState;

impl OutboundSysCall for BotlibAiFreeChatState {
    type Args = BotlibAiFreeChatStateArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_FREE_CHAT_STATE;
}

impl EncodeSysCall for BotlibAiFreeChatState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.handle as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiFreeChatState {
    fn decode_return(_word: isize) -> Self::Output {}
}
