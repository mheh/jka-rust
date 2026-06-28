use core::ffi::c_int;
use crate::ffi::GameImport;
use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_NUM_CONSOLE_MESSAGE` outbound game-to-engine syscall.
///
/// C signature: `int trap_BotNumConsoleMessages(int chatstate)`
/// ABI:         `syscall(BOTLIB_AI_NUM_CONSOLE_MESSAGE, chatstate)`
#[derive(Debug)]
pub struct BotlibAiNumConsoleMessageArgs {
    chatstate: c_int,
}

impl BotlibAiNumConsoleMessageArgs {
    pub fn new(chatstate: c_int) -> Self {
        Self { chatstate }
    }

    pub fn chatstate(&self) -> c_int {
        self.chatstate
    }
}

pub struct BotlibAiNumConsoleMessage;

impl OutboundSysCall for BotlibAiNumConsoleMessage {
    type Import = GameImport;
    type Args = BotlibAiNumConsoleMessageArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_NUM_CONSOLE_MESSAGE;
}

impl EncodeSysCall for BotlibAiNumConsoleMessage {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.chatstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiNumConsoleMessage {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
