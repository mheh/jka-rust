use core::ffi::{c_char, c_int};
use std::ffi::CString;

use super::super::MpUiImport;
use abi_transport::pass_float;

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_TESTPRINTFLOAT` outbound game-to-engine syscall.
///
/// Mirrors `syscall!(UiTESTPRINTFLOAT, string, PASSFLOAT(f))`.
/// The engine ignores both arguments and returns 0; this syscall exists for
/// debug/test instrumentation only.
#[derive(Debug)]
pub struct UiTestprintfloatArgs {
    string: CString,
    f: f32,
}

impl UiTestprintfloatArgs {
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

/// `UI_TESTPRINTFLOAT` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:290`
pub struct UiTestprintfloat;

impl OutboundSysCall for UiTestprintfloat {
    type Import = MpUiImport;
    type Args = UiTestprintfloatArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_TESTPRINTFLOAT;
}

impl EncodeSysCall for UiTestprintfloat {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.string()), pass_float(a.f())])
    }
}

impl DecodeSysCallReturn for UiTestprintfloat {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
