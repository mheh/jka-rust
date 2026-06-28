use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_EA_DELAYED_JUMP` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaDelayedJumpArgs {
    client: c_int,
}

impl BotlibEaDelayedJumpArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

pub struct BotlibEaDelayedJump;

impl OutboundSysCall for BotlibEaDelayedJump {
    type Import = GameImport;
    type Args = BotlibEaDelayedJumpArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_DELAYED_JUMP;
}

impl EncodeSysCall for BotlibEaDelayedJump {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaDelayedJump {
    fn decode_return(_word: isize) -> Self::Output {}
}
