use core::ffi::{c_char, c_int};

use crate::ffi::GameImport;
use super::super::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_STRNCPY` outbound game-to-engine syscall.
///
/// ABI: `strncpy(char *dest, const char *src, int n)` → `int` (dest ptr cast to int)
#[derive(Debug)]
pub struct GStrncpyArgs {
    dest: *mut c_char,
    src: *const c_char,
    n: c_int,
}

impl GStrncpyArgs {
    pub fn new(dest: *mut c_char, src: *const c_char, n: c_int) -> Self {
        Self { dest, src, n }
    }

    pub fn dest(&self) -> *mut c_char {
        self.dest
    }

    pub fn src(&self) -> *const c_char {
        self.src
    }

    pub fn n(&self) -> c_int {
        self.n
    }
}

pub struct GStrncpy;

impl OutboundSysCall for GStrncpy {
    type Args = GStrncpyArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::G_STRNCPY;
}

impl EncodeSysCall for GStrncpy {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(a.dest),
            ptr_to_word(a.src),
            a.n as isize,
        ])
    }
}

impl DecodeSysCallReturn for GStrncpy {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
