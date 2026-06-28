use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for the `BOTLIB_EA_JUMP` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaJumpArgs {
    /// Bot client number.
    client: c_int,
}

impl BotlibEaJumpArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_JUMP` outbound game-to-engine syscall.
pub struct BotlibEaJump;

impl OutboundSysCall for BotlibEaJump {
    type Args = BotlibEaJumpArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_JUMP;
}

impl EncodeSysCall for BotlibEaJump {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaJump {
    fn decode_return(_word: isize) -> Self::Output {}
}
