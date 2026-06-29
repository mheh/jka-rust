use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_CHARACTERISTIC_FLOAT` outbound game-to-engine syscall.
///
/// Reads the float characteristic at `index` for the bot character `character`.
/// C ABI: `float Characteristic_Float(int character, int index)`
#[derive(Debug)]
pub struct BotlibAiCharacteristicFloatArgs {
    character: c_int,
    index: c_int,
}

impl BotlibAiCharacteristicFloatArgs {
    pub fn new(character: c_int, index: c_int) -> Self {
        Self { character, index }
    }

    pub fn character(&self) -> c_int {
        self.character
    }

    pub fn index(&self) -> c_int {
        self.index
    }
}

/// `BOTLIB_AI_CHARACTERISTIC_FLOAT` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:415`
pub struct BotlibAiCharacteristicFloat;

impl OutboundSysCall for BotlibAiCharacteristicFloat {
    type Import = GameImport;
    type Args = BotlibAiCharacteristicFloatArgs;
    type Output = f32;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_CHARACTERISTIC_FLOAT;
}

impl EncodeSysCall for BotlibAiCharacteristicFloat {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.character as isize, a.index as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiCharacteristicFloat {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
