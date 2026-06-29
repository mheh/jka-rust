use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_CHARACTERISTIC_BINTEGER` outbound game-to-engine syscall.
///
/// C: `int trap_Characteristic_BInteger(int character, int index, int min, int max)`
/// ABI: `syscall(BOTLIB_AI_CHARACTERISTIC_BINTEGER, character, index, min, max)`
#[derive(Debug)]
pub struct BotlibAiCharacteristicBintegerArgs {
    character: c_int,
    index: c_int,
    min: c_int,
    max: c_int,
}

impl BotlibAiCharacteristicBintegerArgs {
    pub fn new(character: c_int, index: c_int, min: c_int, max: c_int) -> Self {
        Self {
            character,
            index,
            min,
            max,
        }
    }

    pub fn character(&self) -> c_int {
        self.character
    }
    pub fn index(&self) -> c_int {
        self.index
    }
    pub fn min(&self) -> c_int {
        self.min
    }
    pub fn max(&self) -> c_int {
        self.max
    }
}

/// `BOTLIB_AI_CHARACTERISTIC_BINTEGER` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:418`
pub struct BotlibAiCharacteristicBinteger;

impl OutboundSysCall for BotlibAiCharacteristicBinteger {
    type Import = GameImport;
    type Args = BotlibAiCharacteristicBintegerArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_CHARACTERISTIC_BINTEGER;
}

impl EncodeSysCall for BotlibAiCharacteristicBinteger {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.character as isize,
            a.index as isize,
            a.min as isize,
            a.max as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiCharacteristicBinteger {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
