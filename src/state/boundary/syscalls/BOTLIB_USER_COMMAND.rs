use core::ffi::c_int;
use crate::ffi::GameImport;
use crate::codemp::game::q_shared_h::usercmd_t;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_USER_COMMAND` outbound game-to-engine syscall.
///
/// Feeds the bot at `client_num` the movement command `ucmd`.
/// Mirrors: `syscall!(BOTLIB_USER_COMMAND, client_num, ucmd)`
#[derive(Debug)]
pub struct BotlibUserCommandArgs {
    client_num: c_int,
    ucmd: *mut usercmd_t,
}

impl BotlibUserCommandArgs {
    pub fn new(client_num: c_int, ucmd: *mut usercmd_t) -> Self {
        Self { client_num, ucmd }
    }

    pub fn client_num(&self) -> c_int {
        self.client_num
    }

    pub fn ucmd(&self) -> *mut usercmd_t {
        self.ucmd
    }
}

pub struct BotlibUserCommand;

impl OutboundSysCall for BotlibUserCommand {
    type Args = BotlibUserCommandArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_USER_COMMAND;
}

impl EncodeSysCall for BotlibUserCommand {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.client_num as isize,
            ptr_to_word(a.ucmd),
        ])
    }
}

impl DecodeSysCallReturn for BotlibUserCommand {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
