use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_EA_MOVE_LEFT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaMoveLeftArgs {
    client: c_int,
}

impl BotlibEaMoveLeftArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_MOVE_LEFT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:399`
pub struct BotlibEaMoveLeft;

impl OutboundSysCall for BotlibEaMoveLeft {
    type Import = MpGameImport;
    type Args = BotlibEaMoveLeftArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_MOVE_LEFT;
}

impl EncodeSysCall for BotlibEaMoveLeft {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([(a.client as isize)])
    }
}

impl DecodeSysCallReturn for BotlibEaMoveLeft {
    fn decode_return(_word: isize) -> Self::Output {}
}
