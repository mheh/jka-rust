use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;

use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

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

pub struct GRoffCache;

impl OutboundSysCall for GRoffCache {
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
