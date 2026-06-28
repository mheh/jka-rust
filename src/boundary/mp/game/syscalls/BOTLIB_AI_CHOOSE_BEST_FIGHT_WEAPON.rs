use core::ffi::c_int;

use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_CHOOSE_BEST_FIGHT_WEAPON` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotChooseBestFightWeapon(int weaponstate, int *inventory)`
#[derive(Debug)]
pub struct BotlibAiChooseBestFightWeaponArgs {
    weaponstate: c_int,
    inventory: *mut c_int,
}

impl BotlibAiChooseBestFightWeaponArgs {
    pub fn new(weaponstate: c_int, inventory: *mut c_int) -> Self {
        Self { weaponstate, inventory }
    }

    pub fn weaponstate(&self) -> c_int {
        self.weaponstate
    }

    pub fn inventory(&self) -> *mut c_int {
        self.inventory
    }
}

pub struct BotlibAiChooseBestFightWeapon;

impl OutboundSysCall for BotlibAiChooseBestFightWeapon {
    type Import = GameImport;
    type Args = BotlibAiChooseBestFightWeaponArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_CHOOSE_BEST_FIGHT_WEAPON;
}

impl EncodeSysCall for BotlibAiChooseBestFightWeapon {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.weaponstate as isize,
            ptr_to_word(a.inventory),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiChooseBestFightWeapon {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
