use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_FREE_WEAPON_STATE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiFreeWeaponStateArgs {
    weaponstate: c_int,
}

impl BotlibAiFreeWeaponStateArgs {
    pub fn new(weaponstate: c_int) -> Self {
        Self { weaponstate }
    }

    pub fn weaponstate(&self) -> c_int {
        self.weaponstate
    }
}

/// `BOTLIB_AI_FREE_WEAPON_STATE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:479`
pub struct BotlibAiFreeWeaponState;

impl OutboundSysCall for BotlibAiFreeWeaponState {
    type Import = GameImport;
    type Args = BotlibAiFreeWeaponStateArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_FREE_WEAPON_STATE;
}

impl EncodeSysCall for BotlibAiFreeWeaponState {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([(a.weaponstate as isize)])
    }
}

impl DecodeSysCallReturn for BotlibAiFreeWeaponState {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
