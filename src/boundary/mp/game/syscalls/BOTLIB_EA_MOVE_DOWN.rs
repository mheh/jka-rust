use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_MOVE_DOWN` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaMoveDownArgs {
    client: c_int,
}

impl BotlibEaMoveDownArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_MOVE_DOWN` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:396`
pub struct BotlibEaMoveDown;

impl OutboundSysCall for BotlibEaMoveDown {
    type Import = GameImport;
    type Args = BotlibEaMoveDownArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_MOVE_DOWN;
}

impl EncodeSysCall for BotlibEaMoveDown {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaMoveDown {
    fn decode_return(_word: isize) -> Self::Output {}
}
