use core::ffi::{c_char, c_int};

use crate::ffi::GameImport;

use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_VARIABLEDECLARED` outbound game-to-engine syscall.
#[derive(Debug)]
pub struct GIcarusVariabledeclaredArgs {
    type_name: *const c_char,
}

impl GIcarusVariabledeclaredArgs {
    pub fn new(type_name: *const c_char) -> Self {
        Self { type_name }
    }

    pub fn type_name(&self) -> *const c_char {
        self.type_name
    }
}

pub struct GIcarusVariabledeclared;

impl OutboundSysCall for GIcarusVariabledeclared {
    type Args = GIcarusVariabledeclaredArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_ICARUS_VARIABLEDECLARED;
}

impl EncodeSysCall for GIcarusVariabledeclared {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.type_name)])
    }
}

impl DecodeSysCallReturn for GIcarusVariabledeclared {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
