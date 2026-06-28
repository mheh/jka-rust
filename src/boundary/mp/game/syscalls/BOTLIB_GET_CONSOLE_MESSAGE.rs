use core::ffi::{c_char, c_int};

use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_GET_CONSOLE_MESSAGE` outbound game-to-engine syscall.
///
/// C source: `int trap_BotGetServerCommand(int clientNum, char *message, int size)`
/// ABI:      `syscall(BOTLIB_GET_CONSOLE_MESSAGE, clientNum, message, size)`
#[derive(Debug)]
pub struct BotlibGetConsoleMessageArgs {
    client_num: c_int,
    message: *mut c_char,
    size: c_int,
}

impl BotlibGetConsoleMessageArgs {
    pub fn new(client_num: c_int, message: *mut c_char, size: c_int) -> Self {
        Self { client_num, message, size }
    }

    pub fn client_num(&self) -> c_int { self.client_num }
    pub fn message(&self) -> *mut c_char { self.message }
    pub fn size(&self) -> c_int { self.size }
}

pub struct BotlibGetConsoleMessage;

impl OutboundSysCall for BotlibGetConsoleMessage {
    type Import = GameImport;
    type Args = BotlibGetConsoleMessageArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_GET_CONSOLE_MESSAGE;
}

impl EncodeSysCall for BotlibGetConsoleMessage {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.client_num as isize,
            ptr_to_word(a.message),
            a.size as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibGetConsoleMessage {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
