use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_EA_ACTION` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(BOTLIB_EA_ACTION, client, action)` from `src/trap/ea.rs`.
#[derive(Debug)]
pub struct BotlibEaActionArgs {
    /// Bot client number.
    client: c_int,
    /// Elementary action flags.
    action: c_int,
}

impl BotlibEaActionArgs {
    pub fn new(client: c_int, action: c_int) -> Self {
        Self { client, action }
    }

    pub fn client(&self) -> c_int {
        self.client
    }

    pub fn action(&self) -> c_int {
        self.action
    }
}

pub struct BotlibEaAction;

impl OutboundSysCall for BotlibEaAction {
    type Args = BotlibEaActionArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_ACTION;
}

impl EncodeSysCall for BotlibEaAction {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize, a.action as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaAction {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
