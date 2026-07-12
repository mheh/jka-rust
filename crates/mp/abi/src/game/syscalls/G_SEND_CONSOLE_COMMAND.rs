use core::ffi::{c_char, c_int};
use std::ffi::CString;

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_SEND_CONSOLE_COMMAND` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(G_SEND_CONSOLE_COMMAND, exec_when, c.as_ptr())`.
/// `exec_when` is an engine `EXEC_*` value (0 = now, 1 = insert, 2 = append).
#[derive(Debug)]
pub struct GSendConsoleCommandArgs {
    exec_when: c_int,
    text: CString,
}

impl GSendConsoleCommandArgs {
    pub fn new(exec_when: c_int, text: CString) -> Self {
        Self { exec_when, text }
    }

    pub fn exec_when(&self) -> c_int {
        self.exec_when
    }

    pub fn text(&self) -> *const c_char {
        self.text.as_ptr()
    }
}

/// `G_SEND_CONSOLE_COMMAND` MP game imports syscall ABI token.
///
/// Raven: ( const char *text );
/// Raven: add commands to the console as if they were typed in
/// Raven: for map changing, etc
/// Raven: =========== server specific functionality =============
/// Source: `oracle/codemp/game/g_public.h:138`
pub struct GSendConsoleCommand;

impl OutboundSysCall for GSendConsoleCommand {
    type Import = MpGameImport;
    type Args = GSendConsoleCommandArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_SEND_CONSOLE_COMMAND;
}

impl EncodeSysCall for GSendConsoleCommand {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.exec_when as isize, ptr_to_word(a.text())])
    }
}

impl DecodeSysCallReturn for GSendConsoleCommand {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
