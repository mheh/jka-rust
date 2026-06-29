use core::ffi::{c_char, c_int};
use std::ffi::CString;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport;

/// `BOTLIB_AI_LOAD_CHARACTER` outbound game-to-engine syscall.
///
/// Loads the bot character file `charfile` at the given `skill` level and
/// returns a character handle (`i32`). Models the C ABI directly:
/// `syscall!(BOTLIB_AI_LOAD_CHARACTER, charfile_ptr, pass_float(skill))`.
#[derive(Debug)]
pub struct BotlibAiLoadCharacterArgs {
    charfile: CString,
    skill: f32,
}

impl BotlibAiLoadCharacterArgs {
    pub fn new(charfile: CString, skill: f32) -> Self {
        Self { charfile, skill }
    }

    pub fn charfile(&self) -> *const c_char {
        self.charfile.as_ptr()
    }

    pub fn skill(&self) -> f32 {
        self.skill
    }
}

/// `BOTLIB_AI_LOAD_CHARACTER` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:413`
pub struct BotlibAiLoadCharacter;

impl OutboundSysCall for BotlibAiLoadCharacter {
    type Import = GameImport;
    type Args = BotlibAiLoadCharacterArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_LOAD_CHARACTER;
}

impl EncodeSysCall for BotlibAiLoadCharacter {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.charfile()), pass_float(a.skill())])
    }
}

impl DecodeSysCallReturn for BotlibAiLoadCharacter {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
