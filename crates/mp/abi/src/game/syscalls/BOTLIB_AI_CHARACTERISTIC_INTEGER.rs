use core::ffi::c_int;

use super::super::MpGameImport;

use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

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

/// `BOTLIB_AI_CHARACTERISTIC_INTEGER` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:417`
pub struct BotlibAiCharacteristicInteger;

impl OutboundSysCall for BotlibAiCharacteristicInteger {
    type Import = MpGameImport;
    type Args = BotlibAiCharacteristicIntegerArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_CHARACTERISTIC_INTEGER;
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
