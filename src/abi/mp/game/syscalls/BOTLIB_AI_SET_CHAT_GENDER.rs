use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_SET_CHAT_GENDER` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotSetChatGender(int chatstate, int gender)`
#[derive(Debug)]
pub struct BotlibAiSetChatGenderArgs {
    chatstate: c_int,
    gender: c_int,
}

impl BotlibAiSetChatGenderArgs {
    pub fn new(chatstate: c_int, gender: c_int) -> Self {
        Self { chatstate, gender }
    }

    pub fn chatstate(&self) -> c_int {
        self.chatstate
    }

    pub fn gender(&self) -> c_int {
        self.gender
    }
}

/// `BOTLIB_AI_SET_CHAT_GENDER` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:437`
pub struct BotlibAiSetChatGender;

impl OutboundSysCall for BotlibAiSetChatGender {
    type Import = GameImport;
    type Args = BotlibAiSetChatGenderArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_SET_CHAT_GENDER;
}

impl EncodeSysCall for BotlibAiSetChatGender {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.chatstate as isize, a.gender as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiSetChatGender {
    fn decode_return(_word: isize) -> Self::Output {}
}
