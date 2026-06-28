use core::ffi::c_int;
use crate::ffi::GameImport;
use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_EA_USE` outbound game-to-engine syscall.
///
/// Bot `client` presses the use key.
#[derive(Debug)]
pub struct BotlibEaUseArgs {
    client: c_int,
}

impl BotlibEaUseArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

pub struct BotlibEaUse;

impl OutboundSysCall for BotlibEaUse {
    type Args = BotlibEaUseArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_USE;
}

impl EncodeSysCall for BotlibEaUse {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaUse {
    fn decode_return(_word: isize) -> Self::Output {}
}
