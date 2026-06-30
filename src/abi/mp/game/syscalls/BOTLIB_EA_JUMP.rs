use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

/// `BOTLIB_EA_JUMP` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:403`
pub struct BotlibEaJump;

impl OutboundSysCall for BotlibEaJump {
    type Import = MpGameImport;
    type Args = BotlibEaJumpArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_JUMP;
}

impl EncodeSysCall for BotlibEaJump {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaJump {
    fn decode_return(_word: isize) -> Self::Output {}
}
