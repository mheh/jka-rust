use core::ffi::c_int;

use super::super::MpGameImport;

use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_FREE_CHARACTER` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct BotlibAiFreeCharacterArgs {
    character: c_int,
}

impl BotlibAiFreeCharacterArgs {
    pub fn new(character: c_int) -> Self {
        Self { character }
    }

    pub fn character(&self) -> c_int {
        self.character
    }
}

/// `BOTLIB_AI_FREE_CHARACTER` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:414`
pub struct BotlibAiFreeCharacter;

impl OutboundSysCall for BotlibAiFreeCharacter {
    type Import = MpGameImport;
    type Args = BotlibAiFreeCharacterArgs;
    type Output = ();

    const IMPORT: MpGameImport = MpGameImport::BOTLIB_AI_FREE_CHARACTER;
}

impl EncodeSysCall for BotlibAiFreeCharacter {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.character as isize])
    }
}

impl DecodeSysCallReturn for BotlibAiFreeCharacter {
    fn decode_return(_word: isize) -> Self::Output {}
}
