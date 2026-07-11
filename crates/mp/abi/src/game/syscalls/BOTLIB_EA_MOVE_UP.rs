use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
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

/// `BOTLIB_EA_MOVE_UP` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:395`
pub struct BotlibEaMoveUp;

impl OutboundSysCall for BotlibEaMoveUp {
    type Import = MpGameImport;
    type Args = BotlibEaMoveUpArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_MOVE_UP;
}

impl EncodeSysCall for BotlibEaMoveUp {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([(a.client as isize)])
    }
}

impl DecodeSysCallReturn for BotlibEaMoveUp {
    fn decode_return(_word: isize) -> Self::Output {}
}
