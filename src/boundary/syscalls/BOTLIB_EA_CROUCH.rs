use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for the `BOTLIB_EA_CROUCH` game→engine syscall.
///
/// Mirrors `syscall!(BOTLIB_EA_CROUCH, client)` — bot `client` crouches.
#[derive(Debug)]
pub struct BotlibEaCrouchArgs {
    client: c_int,
}

impl BotlibEaCrouchArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_CROUCH` outbound game-to-engine syscall.
pub struct BotlibEaCrouch;

impl OutboundSysCall for BotlibEaCrouch {
    type Args = BotlibEaCrouchArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_CROUCH;
}

impl EncodeSysCall for BotlibEaCrouch {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([(a.client as isize)])
    }
}

impl DecodeSysCallReturn for BotlibEaCrouch {
    fn decode_return(_word: isize) -> Self::Output {}
}
