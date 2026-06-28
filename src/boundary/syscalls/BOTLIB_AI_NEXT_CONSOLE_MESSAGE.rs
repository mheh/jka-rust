use core::ffi::{c_int, c_void};

use crate::ffi::GameImport;

use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_NEXT_CONSOLE_MESSAGE` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotNextConsoleMessage(int chatstate, void *cm)`
/// Engine fills `*cm` (a `bot_consolemessage_s`) and returns the message handle
/// (0 if none available).
#[derive(Debug)]
pub struct BotlibAiNextConsoleMessageArgs {
    /// Chat-state handle.
    chatstate: c_int,
    /// Caller-allocated `bot_consolemessage_s*`; engine writes through it.
    cm: *mut c_void,
}

impl BotlibAiNextConsoleMessageArgs {
    pub fn new(chatstate: c_int, cm: *mut c_void) -> Self {
        Self { chatstate, cm }
    }

    pub fn chatstate(&self) -> c_int {
        self.chatstate
    }

    pub fn cm(&self) -> *mut c_void {
        self.cm
    }
}

pub struct BotlibAiNextConsoleMessage;

impl OutboundSysCall for BotlibAiNextConsoleMessage {
    type Args = BotlibAiNextConsoleMessageArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_NEXT_CONSOLE_MESSAGE;
}

impl EncodeSysCall for BotlibAiNextConsoleMessage {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.chatstate as isize,
            ptr_to_word(a.cm),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiNextConsoleMessage {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
