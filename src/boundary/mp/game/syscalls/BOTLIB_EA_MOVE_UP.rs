use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_MOVE_UP` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaMoveUpArgs {
    client: c_int,
}

impl BotlibEaMoveUpArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_MOVE_UP` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:395`
pub struct BotlibEaMoveUp;

impl OutboundSysCall for BotlibEaMoveUp {
    type Import = GameImport;
    type Args = BotlibEaMoveUpArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_MOVE_UP;
}

impl EncodeSysCall for BotlibEaMoveUp {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([(a.client as isize)])
    }
}

impl DecodeSysCallReturn for BotlibEaMoveUp {
    fn decode_return(_word: isize) -> Self::Output {}
}
