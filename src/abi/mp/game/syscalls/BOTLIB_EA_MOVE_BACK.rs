use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_MOVE_BACK` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaMoveBackArgs {
    client: c_int,
}

impl BotlibEaMoveBackArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_MOVE_BACK` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:398`
pub struct BotlibEaMoveBack;

impl OutboundSysCall for BotlibEaMoveBack {
    type Import = GameImport;
    type Args = BotlibEaMoveBackArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_MOVE_BACK;
}

impl EncodeSysCall for BotlibEaMoveBack {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaMoveBack {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
