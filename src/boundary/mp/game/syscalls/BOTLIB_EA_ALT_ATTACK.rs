use core::ffi::c_int;
use crate::ffi::GameImport;
use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct BotlibEaAltAttack;

impl OutboundSysCall for BotlibEaAltAttack {
    type Import = GameImport;
    type Args = BotlibEaAltAttackArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_ALT_ATTACK;
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
