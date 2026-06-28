use core::ffi::{c_char, c_int};

use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_ICARUS_GETVECTORVARIABLE` outbound game-to-engine syscall.
///
/// C ABI: `int trap_ICARUS_GetVectorVariable(const char *name, vec3_t *value)`
///
/// The engine reads the named vector variable and writes it into `*value`.
#[derive(Debug)]
pub struct GIcarusGetvectorvariableArgs {
    name: *const c_char,
    value: *mut vec3_t,
}

impl GIcarusGetvectorvariableArgs {
    pub fn new(name: *const c_char, value: *mut vec3_t) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> *const c_char { self.name }
    pub fn value(&self) -> *mut vec3_t { self.value }
}

pub struct GIcarusGetvectorvariable;

impl OutboundSysCall for GIcarusGetvectorvariable {
    type Import = GameImport;
    type Args = GIcarusGetvectorvariableArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_ICARUS_GETVECTORVARIABLE;
}

impl EncodeSysCall for GIcarusGetvectorvariable {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.name()),
            ptr_to_word(a.value()),
        ])
    }
}

impl DecodeSysCallReturn for GIcarusGetvectorvariable {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
