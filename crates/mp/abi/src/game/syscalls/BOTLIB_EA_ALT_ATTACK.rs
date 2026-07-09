use super::super::MpGameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use core::ffi::c_int;

/// `BOTLIB_EA_ALT_ATTACK` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibEaAltAttackArgs {
    client: c_int,
}

impl BotlibEaAltAttackArgs {
    pub fn new(client: c_int) -> Self {
        Self { client }
    }

    pub fn client(&self) -> c_int {
        self.client
    }
}

/// `BOTLIB_EA_ALT_ATTACK` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:390`
pub struct BotlibEaAltAttack;

impl OutboundSysCall for BotlibEaAltAttack {
    type Import = MpGameImport;
    type Args = BotlibEaAltAttackArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_EA_ALT_ATTACK;
}

impl EncodeSysCall for BotlibEaAltAttack {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([(a.client as isize)])
    }
}

impl DecodeSysCallReturn for BotlibEaAltAttack {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
