use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

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

/// `BOTLIB_EA_RESPAWN` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:393`
pub struct BotlibEaRespawn;

impl OutboundSysCall for BotlibEaRespawn {
    type Import = MpGameImport;
    type Args = BotlibEaRespawnArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_RESPAWN;
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
