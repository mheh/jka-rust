use super::super::MpGameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use core::ffi::c_int;

/// `BOTLIB_AI_GET_WEAPON_INFO` outbound game-to-engine syscall.
///
/// Mirrors `trap_BotGetWeaponInfo(int weaponstate, int weapon, void *weaponinfo)`.
/// The engine writes weapon info into the caller-supplied `weaponinfo` out-param.
#[derive(Debug)]
pub struct BotlibAiGetWeaponInfoArgs {
    weaponstate: c_int,
    weapon: c_int,
    weaponinfo: *mut core::ffi::c_void,
}

impl BotlibAiGetWeaponInfoArgs {
    pub fn new(weaponstate: c_int, weapon: c_int, weaponinfo: *mut core::ffi::c_void) -> Self {
        Self {
            weaponstate,
            weapon,
            weaponinfo,
        }
    }

    pub fn weaponstate(&self) -> c_int {
        self.weaponstate
    }
    pub fn weapon(&self) -> c_int {
        self.weapon
    }
    pub fn weaponinfo(&self) -> *mut core::ffi::c_void {
        self.weaponinfo
    }
}

/// `BOTLIB_AI_GET_WEAPON_INFO` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:476`
pub struct BotlibAiGetWeaponInfo;

impl OutboundSysCall for BotlibAiGetWeaponInfo {
    type Import = MpGameImport;
    type Args = BotlibAiGetWeaponInfoArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_GET_WEAPON_INFO;
}

impl EncodeSysCall for BotlibAiGetWeaponInfo {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.weaponstate as isize,
            a.weapon as isize,
            ptr_to_word(a.weaponinfo),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiGetWeaponInfo {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
