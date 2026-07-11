use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_MOVE_RIGHT` outbound game-to-engine syscall.
///
/// Instructs the engine botlib that bot `client` strafes right this frame.
/// Mirrors `syscall!(BOTLIB_EA_MOVE_RIGHT, client)` — one `c_int` argument, void return.
#[derive(Debug)]
pub struct BotlibEaMoveRightArgs {
    /// Bot client number.
    client: c_int,
}

impl BotlibEaMoveRightArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_MOVE_RIGHT` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:400`
pub struct BotlibEaMoveRight;

impl OutboundSysCall for BotlibEaMoveRight {
    type Import = MpGameImport;
    type Args = BotlibEaMoveRightArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_MOVE_RIGHT;
}

impl EncodeSysCall for BotlibEaMoveRight {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaMoveRight {
    fn decode_return(_word: isize) -> Self::Output {}
}
