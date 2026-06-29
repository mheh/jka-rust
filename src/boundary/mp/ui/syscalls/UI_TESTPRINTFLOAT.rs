use core::ffi::{c_char, c_int};
use std::ffi::CString;

use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport;

use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UiTESTPRINTFLOAT` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(UiTESTPRINTFLOAT, string, PASSFLOAT(f))`.
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

/// `UiTESTPRINTFLOAT` MP game imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:290`
pub struct GTestprintfloat;

impl OutboundSysCall for GTestprintfloat {
    type Import = GameImport;
    type Args = GTestprintfloatArgs;
    type Output = c_int;

    const IMPORT: GameImport = GameImport::UiTESTPRINTFLOAT;
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
