use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_REMOVE_CONSOLE_MESSAGE` outbound game-to-engine syscall.
///
/// C ABI: `void trap_BotRemoveConsoleMessage(int chatstate, int handle)`
#[derive(Debug)]
pub struct BotlibAiRemoveConsoleMessageArgs {
    chatstate: c_int,
    handle: c_int,
}

impl BotlibAiRemoveConsoleMessageArgs {
    pub fn new(chatstate: c_int, handle: c_int) -> Self {
        Self { chatstate, handle }
    }

    pub fn chatstate(&self) -> c_int {
        self.chatstate
    }

    pub fn handle(&self) -> c_int {
        self.handle
    }
}

/// `BOTLIB_AI_REMOVE_CONSOLE_MESSAGE` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:424`
pub struct BotlibAiRemoveConsoleMessage;

impl OutboundSysCall for BotlibAiRemoveConsoleMessage {
    type Import = MpGameImport;
    type Args = BotlibAiRemoveConsoleMessageArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_REMOVE_CONSOLE_MESSAGE;
}

impl EncodeSysCall for BotlibAiRemoveConsoleMessage {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.chatstate as isize, a.handle as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiRemoveConsoleMessage {
    fn decode_return(_word: isize) -> Self::Output {}
}
