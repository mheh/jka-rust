use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_EA_RESET_INPUT` outbound game-to-engine syscall.
///
/// Resets the accumulated input for bot `client`.
/// C ABI: `void trap_EA_ResetInput(int client)`
#[derive(Debug)]
pub struct BotlibEaResetInputArgs {
    client: c_int,
}

impl BotlibEaResetInputArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

pub struct BotlibEaResetInput;

impl OutboundSysCall for BotlibEaResetInput {
    type Args = BotlibEaResetInputArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_RESET_INPUT;
}

impl EncodeSysCall for BotlibEaResetInput {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaResetInput {
    fn decode_return(_word: isize) -> Self::Output {}
}
