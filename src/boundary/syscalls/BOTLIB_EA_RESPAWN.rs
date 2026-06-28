use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_EA_RESPAWN` outbound game-to-engine syscall.
///
/// Signals the botlib that bot `client` has respawned.
#[derive(Debug)]
pub struct BotlibEaRespawnArgs {
    client: c_int,
}

impl BotlibEaRespawnArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

pub struct BotlibEaRespawn;

impl OutboundSysCall for BotlibEaRespawn {
    type Args = BotlibEaRespawnArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_RESPAWN;
}

impl EncodeSysCall for BotlibEaRespawn {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaRespawn {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
