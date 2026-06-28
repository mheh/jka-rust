use core::ffi::c_int;
use crate::ffi::GameImport;
use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_RESET_WEAPON_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiResetWeaponStateArgs {
    weaponstate: c_int,
}

impl BotlibAiResetWeaponStateArgs {
    pub fn new(weaponstate: c_int) -> Self {
        Self { weaponstate }
    }

    pub fn weaponstate(&self) -> c_int {
        self.weaponstate
    }
}

pub struct BotlibAiResetWeaponState;

impl OutboundSysCall for BotlibAiResetWeaponState {
    type Import = GameImport;
    type Args = BotlibAiResetWeaponStateArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_RESET_WEAPON_STATE;
}

impl EncodeSysCall for BotlibAiResetWeaponState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.weaponstate as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiResetWeaponState {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
