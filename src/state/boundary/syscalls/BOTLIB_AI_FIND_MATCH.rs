use core::ffi::{c_int, c_void};
use std::ffi::CString;

use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `BOTLIB_AI_FIND_MATCH` outbound game-to-engine syscall.
///
/// C ABI: `int trap_BotFindMatch(char *str, void *match, unsigned long int context)`
#[derive(Debug)]
pub struct BotlibAiFindMatchArgs {
    /// The string to match against bot chat patterns.
    pub str: CString,
    /// Output pointer to a `bot_match_s` struct (engine writes through this).
    pub match_ptr: *mut c_void,
    /// Bitmask context flags (`unsigned long int` in C).
    pub context: u64,
}

impl BotlibAiFindMatchArgs {
    pub fn new(str: CString, match_ptr: *mut c_void, context: u64) -> Self {
        Self { str, match_ptr, context }
    }

    pub fn str(&self) -> &CString { &self.str }
    pub fn match_ptr(&self) -> *mut c_void { self.match_ptr }
    pub fn context(&self) -> u64 { self.context }
}

pub struct BotlibAiFindMatch;

impl OutboundSysCall for BotlibAiFindMatch {
    type Args = BotlibAiFindMatchArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::BOTLIB_AI_FIND_MATCH;
}

impl EncodeSysCall for BotlibAiFindMatch {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.str.as_ptr()),
            ptr_to_word(a.match_ptr),
            a.context as isize,
        ])
    }
}

impl DecodeSysCallReturn for BotlibAiFindMatch {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
