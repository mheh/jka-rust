use core::ffi::c_int;

use crate::ffi::GameImport;

use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_CHARACTERISTIC_STRING` outbound game-to-engine syscall.
///
/// C: `void trap_Characteristic_String(int character, int index, char *buf, int size)`
#[derive(Debug)]
pub struct BotlibAiCharacteristicStringArgs {
    character: c_int,
    index: c_int,
    buf: *mut u8,
    size: c_int,
}

impl BotlibAiCharacteristicStringArgs {
    pub fn new(character: c_int, index: c_int, buf: *mut u8, size: c_int) -> Self {
        Self { character, index, buf, size }
    }

    pub fn character(&self) -> c_int { self.character }
    pub fn index(&self) -> c_int { self.index }
    pub fn buf(&self) -> *mut u8 { self.buf }
    pub fn size(&self) -> c_int { self.size }
}

pub struct BotlibAiCharacteristicString;

impl OutboundSysCall for BotlibAiCharacteristicString {
    type Args = BotlibAiCharacteristicStringArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_CHARACTERISTIC_STRING;
}

impl EncodeSysCall for BotlibAiCharacteristicString {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.character as isize,
            a.index as isize,
            ptr_to_word(a.buf),
            a.size as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiCharacteristicString {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
