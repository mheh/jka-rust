use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_TALK` outbound game-to-engine syscall.
///
/// Signals the engine that bot `client` enters talk mode.
#[derive(Debug)]
pub struct BotlibEaTalkArgs {
    client: c_int,
}

impl BotlibEaTalkArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_TALK` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:388`
pub struct BotlibEaTalk;

impl OutboundSysCall for BotlibEaTalk {
    type Import = GameImport;
    type Args = BotlibEaTalkArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_TALK;
}

impl EncodeSysCall for BotlibEaTalk {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaTalk {
    fn decode_return(_word: isize) -> Self::Output {}
}
