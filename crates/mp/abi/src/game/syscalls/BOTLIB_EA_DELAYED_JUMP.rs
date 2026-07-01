use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

/// `BOTLIB_EA_DELAYED_JUMP` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:404`
pub struct BotlibEaDelayedJump;

impl OutboundSysCall for BotlibEaDelayedJump {
    type Import = MpGameImport;
    type Args = BotlibEaDelayedJumpArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_DELAYED_JUMP;
}

impl EncodeSysCall for BotlibEaDelayedJump {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaDelayedJump {
    fn decode_return(_word: isize) -> Self::Output {}
}
