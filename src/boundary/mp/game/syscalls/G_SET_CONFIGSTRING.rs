use core::ffi::c_int;
use std::ffi::CString;

use crate::ffi::GameImport;
use crate::boundary::generic::{ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// `G_SET_CONFIGSTRING` outbound game-to-engine syscall.
///
/// Mirrors the C ABI: `syscall(G_SET_CONFIGSTRING, num, string)` → void.
#[derive(Debug)]
pub struct GSetConfigstringArgs {
    /// Configstring index.
    num: c_int,
    /// Null-terminated value string.
    string: CString,
}

impl GSetConfigstringArgs {
    pub fn new(num: c_int, string: CString) -> Self {
        Self { num, string }
    }

    pub fn num(&self) -> c_int {
        self.num
    }

    pub fn string(&self) -> &CString {
        &self.string
    }
}

pub struct GSetConfigstring;

impl OutboundSysCall for GSetConfigstring {
    type Import = GameImport;
    type Args = GSetConfigstringArgs;
    type Output = ();

    const IMPORT: GameImport = GameImport::G_SET_CONFIGSTRING;
}

impl EncodeSysCall for GSetConfigstring {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            a.num as isize,
            ptr_to_word(a.string.as_ptr()),
        ])
    }
}

impl DecodeSysCallReturn for GSetConfigstring {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
