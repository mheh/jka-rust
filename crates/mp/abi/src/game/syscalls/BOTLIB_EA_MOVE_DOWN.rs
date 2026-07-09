use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

/// `BOTLIB_EA_MOVE_DOWN` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:396`
pub struct BotlibEaMoveDown;

impl OutboundSysCall for BotlibEaMoveDown {
    type Import = MpGameImport;
    type Args = BotlibEaMoveDownArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_MOVE_DOWN;
}

impl EncodeSysCall for BotlibEaMoveDown {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaMoveDown {
    fn decode_return(_word: isize) -> Self::Output {}
}
