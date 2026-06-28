use core::ffi::c_char;
use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_UNIFY_WHITE_SPACES` outbound game-to-engine syscall.
///
/// C ABI: `void trap_UnifyWhiteSpaces(char *string)`
#[derive(Debug)]
pub struct BotlibAiUnifyWhiteSpacesArgs {
    string: *mut c_char,
}

impl BotlibAiUnifyWhiteSpacesArgs {
    pub fn new(string: *mut c_char) -> Self {
        Self { string }
    }

    pub fn string(&self) -> *mut c_char {
        self.string
    }
}

pub struct BotlibAiUnifyWhiteSpaces;

impl OutboundSysCall for BotlibAiUnifyWhiteSpaces {
    type Args = BotlibAiUnifyWhiteSpacesArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::BOTLIB_AI_UNIFY_WHITE_SPACES;
}

impl EncodeSysCall for BotlibAiUnifyWhiteSpaces {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.string)])
    }
}

impl DecodeSysCallReturn for BotlibAiUnifyWhiteSpaces {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
