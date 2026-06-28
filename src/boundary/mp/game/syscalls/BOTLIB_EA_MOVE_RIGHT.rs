use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct BotlibEaMoveRight;

impl OutboundSysCall for BotlibEaMoveRight {
    type Import = GameImport;
    type Args = BotlibEaMoveRightArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_MOVE_RIGHT;
}

impl EncodeSysCall for BotlibEaMoveRight {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaMoveRight {
    fn decode_return(_word: isize) -> Self::Output {}
}
