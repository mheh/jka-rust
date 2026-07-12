use super::super::MpGameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use core::ffi::c_int;
use mp_qshared::common::mp::qcommon::usercmd_t;

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

/// `BOTLIB_USER_COMMAND` MP game imports syscall ABI token.
///
/// Raven: ( int client, usercmd_t *ucmd );
/// Source: `oracle/codemp/game/g_public.h:354`
pub struct BotlibUserCommand;

impl OutboundSysCall for BotlibUserCommand {
    type Import = MpGameImport;
    type Args = BotlibUserCommandArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_USER_COMMAND;
}

impl EncodeSysCall for BotlibUserCommand {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client_num as isize, ptr_to_word(a.ucmd)])
    }
}

impl DecodeSysCallReturn for BotlibUserCommand {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
