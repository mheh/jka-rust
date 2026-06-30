use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::abi::pass_float;

/// Arguments for `CG_R_FONT_STRLENPIXELS`.
///
/// Raven wrapper: `return syscall( CG_R_FONT_STRLENPIXELS, text, iFontIndex, PASSFLOAT(scale));`
/// Raven transport: `return re.Font_StrLenPixels( (const char *)VMA(1), args[2], VMF(3) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:289-291`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2275`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:873-874`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRFontStrlenpixelsArgs {
    text: *const c_char,
    i_font_index: c_int,
    scale: f32,
}

impl CgRFontStrlenpixelsArgs {
    pub const fn new(text: *const c_char, i_font_index: c_int, scale: f32) -> Self {
        Self {
            text,
            i_font_index,
            scale,
        }
    }

    pub const fn scale(&self) -> f32 {
        self.scale
    }
}

/// `CG_R_FONT_STRLENPIXELS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:122`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:289-291`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:873-874`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:873-874`
pub struct CgRFontStrlenpixels;

impl OutboundSysCall for CgRFontStrlenpixels {
    type Import = MpCgameImport;
    type Args = CgRFontStrlenpixelsArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_FONT_STRLENPIXELS;
}

impl EncodeSysCall for CgRFontStrlenpixels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.text),
            args.i_font_index as isize,
            pass_float(args.scale()),
        ])
    }
}

impl DecodeSysCallReturn for CgRFontStrlenpixels {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
