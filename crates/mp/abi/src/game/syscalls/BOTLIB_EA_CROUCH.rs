use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

/// `BOTLIB_EA_CROUCH` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:394`
pub struct BotlibEaCrouch;

impl OutboundSysCall for BotlibEaCrouch {
    type Import = MpGameImport;
    type Args = BotlibEaCrouchArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_CROUCH;
}

impl EncodeSysCall for BotlibEaCrouch {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([(a.client as isize)])
    }
}

impl DecodeSysCallReturn for BotlibEaCrouch {
    fn decode_return(_word: isize) -> Self::Output {}
}
