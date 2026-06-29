use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `UI_R_FONT_DRAWSTRING`.
///
/// Raven wrapper: `syscall( UI_R_FONT_DRAWSTRING, ox, oy, text, rgba, setIndex, iCharLimit, PASSFLOAT(scale));`
/// Raven transport: `re.Font_DrawString( args[1], args[2], (const char *)VMA(3), (const float *) VMA(4), args[5], args[6], VMF(7) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:131-133`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:996`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1144-1146`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRFontDrawstringArgs {
    ox: c_int,
    oy: c_int,
    text: *const c_char,
    rgba: *const f32,
    set_index: c_int,
    i_char_limit: c_int,
    scale: f32,
}

impl UiRFontDrawstringArgs {
    pub const fn new(
        ox: c_int,
        oy: c_int,
        text: *const c_char,
        rgba: *const f32,
        set_index: c_int,
        i_char_limit: c_int,
        scale: f32,
    ) -> Self {
        Self {
            ox,
            oy,
            text,
            rgba,
            set_index,
            i_char_limit,
            scale,
        }
    }
}

/// `UI_R_FONT_DRAWSTRING` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:79`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:131-133`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1144-1146`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1144-1146`
pub struct UiRFontDrawstring;

impl OutboundSysCall for UiRFontDrawstring {
    type Import = MpUiImport;
    type Args = UiRFontDrawstringArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_R_FONT_DRAWSTRING;
}

impl EncodeSysCall for UiRFontDrawstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.ox as isize,
            args.oy as isize,
            ptr_to_word(args.text),
            ptr_to_word(args.rgba),
            args.set_index as isize,
            args.i_char_limit as isize,
            pass_float(args.scale),
        ])
    }
}

impl DecodeSysCallReturn for UiRFontDrawstring {
    fn decode_return(_word: isize) -> Self::Output {}
}
