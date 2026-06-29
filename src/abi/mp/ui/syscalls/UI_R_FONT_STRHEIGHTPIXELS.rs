use core::ffi::c_int;

use super::super::MpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `UI_R_FONT_STRHEIGHTPIXELS`.
///
/// C ABI: `int trap_R_Font_HeightPixels(const int iFontIndex, const float scale)`.
/// Raven's client switch reads the font index from `args[1]` and packs the
/// scale as a float word.
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:126-128`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:126-128`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1141-1142`
#[derive(Debug, Clone, Copy)]
pub struct UiRFontStrheightpixelsArgs {
    pub font_index: c_int,
    pub scale: f32,
}

impl UiRFontStrheightpixelsArgs {
    pub const fn new(font_index: c_int, scale: f32) -> Self {
        Self { font_index, scale }
    }
}

/// `UI_R_FONT_STRHEIGHTPIXELS` MP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:78`
pub struct UiRFontStrheightpixels;

impl OutboundSysCall for UiRFontStrheightpixels {
    type Import = MpUiImport;
    type Args = UiRFontStrheightpixelsArgs;
    type Output = c_int;

    const IMPORT: MpUiImport = MpUiImport::UI_R_FONT_STRHEIGHTPIXELS;
}

impl EncodeSysCall for UiRFontStrheightpixels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.font_index as isize, pass_float(args.scale)])
    }
}

impl DecodeSysCallReturn for UiRFontStrheightpixels {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
