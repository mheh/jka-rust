use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ROFF_CACHE` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GRoffCacheArgs {
    file: CString,
}

impl GRoffCacheArgs {
    pub fn new(file: CString) -> Self {
        Self { file }
    }

    pub fn file(&self) -> &CString {
        &self.file
    }
}

/// `G_ROFF_CACHE` MP game imports syscall ABI token.
///
/// Raven: int		ROFF_Cache(char *file);
/// Source: `oracle/oracle/codemp/game/g_public.h:243`
pub struct GRoffCache;

impl OutboundSysCall for GRoffCache {
    type Import = GameImport;
    type Args = GRoffCacheArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_ROFF_CACHE;
}

impl EncodeSysCall for GRoffCache {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.file.as_ptr())])
    }
}

impl DecodeSysCallReturn for GRoffCache {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
