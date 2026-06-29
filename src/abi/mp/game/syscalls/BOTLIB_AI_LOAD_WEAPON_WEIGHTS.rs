use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_LOAD_WEAPON_WEIGHTS` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiLoadWeaponWeightsArgs {
    weaponstate: c_int,
    filename: CString,
}

impl BotlibAiLoadWeaponWeightsArgs {
    pub fn new(weaponstate: c_int, filename: CString) -> Self {
        Self {
            weaponstate,
            filename,
        }
    }

    pub fn weaponstate(&self) -> c_int {
        self.weaponstate
    }

    pub fn filename(&self) -> &CString {
        &self.filename
    }
}

/// `BOTLIB_AI_LOAD_WEAPON_WEIGHTS` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:477`
pub struct BotlibAiLoadWeaponWeights;

impl OutboundSysCall for BotlibAiLoadWeaponWeights {
    type Import = GameImport;
    type Args = BotlibAiLoadWeaponWeightsArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_LOAD_WEAPON_WEIGHTS;
}

impl EncodeSysCall for BotlibAiLoadWeaponWeights {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.weaponstate as isize, ptr_to_word(a.filename.as_ptr())])
    }
}

impl DecodeSysCallReturn for BotlibAiLoadWeaponWeights {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
