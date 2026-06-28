use core::ffi::c_int;

use crate::ffi::GameImport;

use crate::boundary::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_CHARACTERISTIC_INTEGER` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiCharacteristicIntegerArgs {
    character: c_int,
    index: c_int,
}

impl BotlibAiCharacteristicIntegerArgs {
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

pub struct BotlibAiCharacteristicInteger;

impl OutboundSysCall for BotlibAiCharacteristicInteger {
    type Import = GameImport;
    type Args = BotlibAiCharacteristicIntegerArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_CHARACTERISTIC_INTEGER;
}

impl EncodeSysCall for BotlibAiCharacteristicInteger {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.character as isize, a.index as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiCharacteristicInteger {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
