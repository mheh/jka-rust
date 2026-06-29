use core::ffi::{c_char, c_int};

use crate::ffi::GameImport;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_ICARUS_GETFLOATVARIABLE` outbound game-to-engine syscall.
///
/// Reads a float ICARUS variable named `name` into the caller-allocated `value` slot.
/// Returns non-zero on success.
#[derive(Debug)]
pub struct GIcarusGetfloatvariableArgs {
    name: *const c_char,
    value: *mut f32,
}

impl GIcarusGetfloatvariableArgs {
    pub fn new(name: *const c_char, value: *mut f32) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> *const c_char {
        self.name
    }

    pub fn value(&self) -> *mut f32 {
        self.value
    }
}

/// `G_ICARUS_GETFLOATVARIABLE` MP game imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/game/g_public.h:269`
pub struct GIcarusGetfloatvariable;

impl OutboundSysCall for GIcarusGetfloatvariable {
    type Import = GameImport;
    type Args = GIcarusGetfloatvariableArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_ICARUS_GETFLOATVARIABLE;
}

impl EncodeSysCall for GIcarusGetfloatvariable {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.name), ptr_to_word(a.value)])
    }
}

impl DecodeSysCallReturn for GIcarusGetfloatvariable {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
