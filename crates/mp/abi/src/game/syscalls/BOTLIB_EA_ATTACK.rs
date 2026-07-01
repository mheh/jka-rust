use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_EA_ATTACK` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaAttackArgs {
    client: c_int,
}

impl BotlibEaAttackArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_ATTACK` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:389`
pub struct BotlibEaAttack;

impl OutboundSysCall for BotlibEaAttack {
    type Import = MpGameImport;
    type Args = BotlibEaAttackArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_ATTACK;
}

impl EncodeSysCall for BotlibEaAttack {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaAttack {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
