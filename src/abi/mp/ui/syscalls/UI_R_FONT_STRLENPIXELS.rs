use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::abi::pass_float;

/// Arguments for `UI_R_FONT_STRLENPIXELS`.
///
/// C ABI: `int trap_R_Font_StrLenPixels(const char *text, const int iFontIndex, const float scale)`.
/// Raven's client switch forwards `text` through `VMA(1)`, reads the font
/// index from `args[2]`, and packs the scale as a float word.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:116-118`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:116-118`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1135-1136`
#[derive(Debug, Clone, Copy)]
pub struct UiRFontStrlenpixelsArgs {
    pub text: *const c_char,
    pub font_index: c_int,
    pub scale: f32,
}

impl UiRFontStrlenpixelsArgs {
    pub const fn new(text: *const c_char, font_index: c_int, scale: f32) -> Self {
        Self {
            text,
            font_index,
            scale,
        }
    }
}

/// `UI_R_FONT_STRLENPIXELS` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:76`
pub struct UiRFontStrlenpixels;

impl OutboundSysCall for UiRFontStrlenpixels {
    type Import = MpUiImport;
    type Args = UiRFontStrlenpixelsArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_R_FONT_STRLENPIXELS;
}

impl EncodeSysCall for UiRFontStrlenpixels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.text),
            args.font_index as isize,
            pass_float(args.scale),
        ])
    }
}

impl DecodeSysCallReturn for UiRFontStrlenpixels {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
