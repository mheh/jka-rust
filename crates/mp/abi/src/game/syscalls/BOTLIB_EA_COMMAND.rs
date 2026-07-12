use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpGameImport;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_COMMAND` outbound game-to-engine syscall.
///
/// Bot `client` issues a console `command` string to the engine.
/// Mirrors: `syscall!(BOTLIB_EA_COMMAND, client, c.as_ptr())`
#[derive(Debug)]
pub struct BotlibEaCommandArgs {
    /// Client number of the bot.
    client: c_int,
    /// Null-terminated command string (caller retains ownership for lifetime of call).
    command: CString,
}

impl BotlibEaCommandArgs {
    pub fn new(client: c_int, command: CString) -> Self {
        Self { client, command }
    }

    pub fn client(&self) -> c_int {
        self.client
    }

    pub fn command(&self) -> &CString {
        &self.command
    }
}

/// `BOTLIB_EA_COMMAND` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:384`
pub struct BotlibEaCommand;

impl OutboundSysCall for BotlibEaCommand {
    type Import = MpGameImport;
    type Args = BotlibEaCommandArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_COMMAND;
}

impl EncodeSysCall for BotlibEaCommand {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize, ptr_to_word(a.command.as_ptr())])
    }
}

impl DecodeSysCallReturn for BotlibEaCommand {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
