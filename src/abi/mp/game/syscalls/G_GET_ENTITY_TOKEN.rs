use core::ffi::{c_char, c_int};

use crate::{ffi::GameImport, shared::qboolean};

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_GET_ENTITY_TOKEN` outbound game-to-engine syscall.
///
/// Pulls the next token from the BSP entity string the engine cached at load,
/// writing it into the caller-owned `buffer`.  Returns `qtrue` while tokens
/// remain, `qfalse` at end of string.  Mirrors the C ABI exactly:
/// `( char *buffer, int bufferSize ) -> qboolean`.
#[derive(Debug)]
pub struct GGetEntityTokenArgs {
    /// Caller-owned output buffer; the engine writes the token here.
    buffer: *mut c_char,
    /// Length of `buffer` in bytes.
    buffer_size: c_int,
}

impl GGetEntityTokenArgs {
    pub fn new(buffer: *mut c_char, buffer_size: c_int) -> Self {
        Self {
            buffer,
            buffer_size,
        }
    }

    pub fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub fn buffer_size(&self) -> c_int {
        self.buffer_size
    }
}

/// `G_GET_ENTITY_TOKEN` MP game imports syscall ABI token.
///
/// Raven: qboolean ( char *buffer, int bufferSize )
/// Raven: Retrieves the next string token from the entity spawn text, returning
/// Raven: false when all tokens have been parsed.
/// Raven: This should only be done at GAME_INIT time.
/// Source: `oracle/oracle/codemp/game/g_public.h:221`
pub struct GGetEntityToken;

impl OutboundSysCall for GGetEntityToken {
    type Import = GameImport;
    type Args = GGetEntityTokenArgs;
    type Output = qboolean;

    const IMPORT: GameImport = GameImport::G_GET_ENTITY_TOKEN;
}

impl EncodeSysCall for GGetEntityToken {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.buffer), a.buffer_size as isize])
    }
}

impl DecodeSysCallReturn for GGetEntityToken {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
