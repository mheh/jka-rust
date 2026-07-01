use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_EA_FORCEPOWER` outbound game-to-engine syscall.
///
/// Signals the bot library that bot `client` should use a force power this frame.
/// Mirrors `trap_EA_ForcePower(client: i32)` / `syscall!(BOTLIB_EA_FORCEPOWER, client)`.
#[derive(Debug)]
pub struct BotlibEaForcepowerArgs {
    /// Bot client number.
    client: c_int,
}

impl BotlibEaForcepowerArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_FORCEPOWER` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:391`
pub struct BotlibEaForcepower;

impl OutboundSysCall for BotlibEaForcepower {
    type Import = MpGameImport;
    type Args = BotlibEaForcepowerArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_FORCEPOWER;
}

impl EncodeSysCall for BotlibEaForcepower {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([(a.client as isize)])
    }
}

impl DecodeSysCallReturn for BotlibEaForcepower {
    fn decode_return(_word: isize) -> Self::Output {}
}
