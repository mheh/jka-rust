use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_R_FONT_STRLENCHARS`.
///
/// C ABI: `int trap_R_Font_StrLenChars(const char *text)`.
/// Raven's client switch forwards the text through `VMA(1)` and returns the
/// string length word.
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:121-123`
/// Output source: `oracle/codemp/ui/ui_syscalls.c:121-123`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1138-1139`
#[derive(Debug, Clone, Copy)]
pub struct UiRFontStrlencharsArgs {
    pub text: *const c_char,
}

impl UiRFontStrlencharsArgs {
    pub const fn new(text: *const c_char) -> Self {
        Self { text }
    }
}

/// `UI_R_FONT_STRLENCHARS` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:77`
pub struct UiRFontStrlenchars;

impl OutboundSysCall for UiRFontStrlenchars {
    type Import = MpUiImport;
    type Args = UiRFontStrlencharsArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_R_FONT_STRLENCHARS;
}

impl EncodeSysCall for UiRFontStrlenchars {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.text)])
    }
}

impl DecodeSysCallReturn for UiRFontStrlenchars {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
