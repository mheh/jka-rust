use core::ffi::c_int;

use super::super::MpGameImport;
use abi_transport::pass_float;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `BOTLIB_AI_CHARACTERISTIC_BFLOAT` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiCharacteristicBfloatArgs {
    character: c_int,
    index: c_int,
    min: f32,
    max: f32,
}

impl BotlibAiCharacteristicBfloatArgs {
    pub fn new(character: c_int, index: c_int, min: f32, max: f32) -> Self {
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
    pub fn min(&self) -> f32 {
        self.min
    }
    pub fn max(&self) -> f32 {
        self.max
    }
}

/// `BOTLIB_AI_CHARACTERISTIC_BFLOAT` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:416`
pub struct BotlibAiCharacteristicBfloat;

impl OutboundSysCall for BotlibAiCharacteristicBfloat {
    type Import = MpGameImport;
    type Args = BotlibAiCharacteristicBfloatArgs;
    type Output = f32;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_CHARACTERISTIC_BFLOAT;
}

impl EncodeSysCall for BotlibAiCharacteristicBfloat {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.character as isize,
            a.index as isize,
            pass_float(a.min),
            pass_float(a.max),
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiCharacteristicBfloat {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
