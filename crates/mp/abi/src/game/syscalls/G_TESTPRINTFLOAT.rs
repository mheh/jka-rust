use core::ffi::{c_char, c_int};
use std::ffi::CString;

use super::super::MpGameImport;
use abi_transport::pass_float;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `G_TESTPRINTFLOAT` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(G_TESTPRINTFLOAT, string, PASSFLOAT(f))`.
/// The engine ignores both arguments and returns 0; this syscall exists for
/// debug/test instrumentation only.
#[derive(Debug)]
pub struct GTestprintfloatArgs {
    string: CString,
    f: f32,
}

impl GTestprintfloatArgs {
    pub fn new(string: CString, f: f32) -> Self {
        Self { string, f }
    }

    pub fn string(&self) -> *const c_char {
        self.string.as_ptr()
    }

    pub fn f(&self) -> f32 {
        self.f
    }
}

/// `G_TESTPRINTFLOAT` MP game imports syscall ABI token.
///
/// Source: `oracle/codemp/game/g_public.h:290`
pub struct GTestprintfloat;

impl OutboundSysCall for GTestprintfloat {
    type Import = MpGameImport;
    type Args = GTestprintfloatArgs;
    type Output = c_int;

    const IMPORT: MpGameImport = MpGameImport::G_TESTPRINTFLOAT;
}

impl EncodeSysCall for GTestprintfloat {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.string()), pass_float(a.f())])
    }
}

impl DecodeSysCallReturn for GTestprintfloat {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
