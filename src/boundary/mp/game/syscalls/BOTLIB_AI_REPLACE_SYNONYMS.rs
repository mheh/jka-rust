use core::ffi::{c_char, c_ulong};

use crate::ffi::GameImport;

use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_REPLACE_SYNONYMS` outbound game-to-engine syscall.
///
/// C: `void trap_BotReplaceSynonyms(char *string, unsigned long int context)`
#[derive(Debug)]
pub struct BotlibAiReplaceSynonymsArgs {
    /// Mutable string buffer whose synonyms will be replaced in-place.
    pub string: *mut c_char,
    /// Synonym context mask.
    pub context: c_ulong,
}

impl BotlibAiReplaceSynonymsArgs {
    pub fn new(string: *mut c_char, context: c_ulong) -> Self {
        Self { string, context }
    }

    pub fn string(&self) -> *mut c_char {
        self.string
    }

    pub fn context(&self) -> c_ulong {
        self.context
    }
}

pub struct BotlibAiReplaceSynonyms;

impl OutboundSysCall for BotlibAiReplaceSynonyms {
    type Import = GameImport;
    type Args = BotlibAiReplaceSynonymsArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_REPLACE_SYNONYMS;
}

impl EncodeSysCall for BotlibAiReplaceSynonyms {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.string as *const _),
            a.context as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiReplaceSynonyms {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
