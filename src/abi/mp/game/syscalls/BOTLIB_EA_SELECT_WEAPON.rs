use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_EA_SELECT_WEAPON` outbound game-to-engine syscall.
///
/// Instructs the bot engine that bot `client` selects `weapon`.
/// Mirrors `syscall!(BOTLIB_EA_SELECT_WEAPON, client, weapon)`.
#[derive(Debug)]
pub struct BotlibEaSelectWeaponArgs {
    /// Bot client number.
    client: c_int,
    /// Weapon index to select.
    weapon: c_int,
}

impl BotlibEaSelectWeaponArgs {
    pub fn new(client: c_int, weapon: c_int) -> Self {
        Self { client, weapon }
    }

    pub fn client(&self) -> c_int {
        self.client
    }

    pub fn weapon(&self) -> c_int {
        self.weapon
    }
}

/// `BOTLIB_EA_SELECT_WEAPON` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:402`
pub struct BotlibEaSelectWeapon;

impl OutboundSysCall for BotlibEaSelectWeapon {
    type Import = GameImport;
    type Args = BotlibEaSelectWeaponArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_EA_SELECT_WEAPON;
}

impl EncodeSysCall for BotlibEaSelectWeapon {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.client as isize, a.weapon as isize])
    }
}

impl DecodeSysCallReturn for BotlibEaSelectWeapon {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
