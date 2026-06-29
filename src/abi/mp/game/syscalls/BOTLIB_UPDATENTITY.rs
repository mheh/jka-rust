use core::ffi::{c_int, c_void};

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::GameImport;

/// `BOTLIB_UPDATENTITY` outbound game-to-engine syscall.
///
/// Pushes entity `ent`'s updated state (via an opaque `bot_updateentity_s *`)
/// into the bot library.
#[derive(Debug)]
pub struct BotlibUpdatentityArgs {
    /// Entity number being updated.
    ent: c_int,
    /// Pointer to a `bot_updateentity_s` (opaque to the game module).
    bue: *mut c_void,
}

impl BotlibUpdatentityArgs {
    pub fn new(ent: c_int, bue: *mut c_void) -> Self {
        Self { ent, bue }
    }

    pub fn ent(&self) -> c_int {
        self.ent
    }

    pub fn bue(&self) -> *mut c_void {
        self.bue
    }
}

/// `BOTLIB_UPDATENTITY` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:349`
pub struct BotlibUpdatentity;

impl OutboundSysCall for BotlibUpdatentity {
    type Import = GameImport;
    type Args = BotlibUpdatentityArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_UPDATENTITY;
}

impl EncodeSysCall for BotlibUpdatentity {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([a.ent as isize, ptr_to_word(a.bue)])
    }
}

impl DecodeSysCallReturn for BotlibUpdatentity {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
