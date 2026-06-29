use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;
use core::ffi::c_char;

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

/// `BOTLIB_AI_UNIFY_WHITE_SPACES` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:434`
pub struct BotlibAiUnifyWhiteSpaces;

impl OutboundSysCall for BotlibAiUnifyWhiteSpaces {
    type Import = GameImport;
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
