use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_SEND_SERVER_COMMAND` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(G_SEND_SERVER_COMMAND, client_num, c.as_ptr())`.
/// `client_num == -1` broadcasts to all clients.
#[derive(Debug)]
pub struct GSendServerCommandArgs {
    /// Client number to target; -1 = broadcast.
    client_num: c_int,
    /// Null-terminated command string.
    text: CString,
}

impl GSendServerCommandArgs {
    pub fn new(client_num: c_int, text: CString) -> Self {
        Self { client_num, text }
    }

    pub fn client_num(&self) -> c_int {
        self.client_num
    }

    pub fn text(&self) -> &CString {
        &self.text
    }
}

/// `G_SEND_SERVER_COMMAND` MP game imports syscall ABI token.
///
/// Raven: ( int clientNum, const char *fmt, ... );
/// Raven: reliably sends a command string to be interpreted by the given
/// Raven: client.  If clientNum is -1, it will be sent to all clients
/// Source: `oracle/codemp/game/g_public.h:153`
pub struct GSendServerCommand;

impl OutboundSysCall for GSendServerCommand {
    type Import = MpGameImport;
    type Args = GSendServerCommandArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::G_SEND_SERVER_COMMAND;
}

impl EncodeSysCall for GSendServerCommand {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client_num as isize, ptr_to_word(a.text.as_ptr())])
    }
}

impl DecodeSysCallReturn for GSendServerCommand {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
