use core::ffi::{c_char, c_int};

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ICARUS_GETSTRINGVARIABLE` outbound game-to-engine syscall.
///
/// Reads string variable `name` into the caller's `value` buffer.
/// The C ABI types the out-buffer `const char *` (engine writes through it — faithful quirk).
#[derive(Debug)]
pub struct GIcarusGetstringvariableArgs {
    /// Variable name to look up.
    name: *const c_char,
    /// Caller-provided output buffer (engine writes the string into it).
    value: *const c_char,
}

impl GIcarusGetstringvariableArgs {
    pub fn new(name: *const c_char, value: *const c_char) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> *const c_char {
        self.name
    }

    pub fn value(&self) -> *const c_char {
        self.value
    }
}

/// `G_ICARUS_GETSTRINGVARIABLE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:270`
pub struct GIcarusGetstringvariable;

impl OutboundSysCall for GIcarusGetstringvariable {
    type Import = GameImport;
    type Args = GIcarusGetstringvariableArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_ICARUS_GETSTRINGVARIABLE;
}

impl EncodeSysCall for GIcarusGetstringvariable {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.name), ptr_to_word(a.value)])
    }
}

impl DecodeSysCallReturn for GIcarusGetstringvariable {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
