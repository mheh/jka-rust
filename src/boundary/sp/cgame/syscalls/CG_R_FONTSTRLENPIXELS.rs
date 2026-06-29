use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `CG_R_FONTSTRLENPIXELS`.
///
/// Raven wrapper: `return syscall( CG_R_FONTSTRLENPIXELS, text, iFontIndex, PASSFLOAT(scale) );`
/// Raven transport: `return re.Font_StrLenPixels((const char *) VMA(1), args[2], VMF(3));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:325-326`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:665-666`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRFontstrlenpixelsArgs {
    text: *const c_char,
    i_font_index: c_int,
    scale: f32,
}

impl CgRFontstrlenpixelsArgs {
    pub const fn new(text: *const c_char, i_font_index: c_int, scale: f32) -> Self {
        Self {
            text,
            i_font_index,
            scale,
        }
    }

    pub const fn text(&self) -> *const c_char {
        self.text
    }

    pub const fn i_font_index(&self) -> c_int {
        self.i_font_index
    }

    pub const fn scale(&self) -> f32 {
        self.scale
    }
}

/// `CG_R_FONTSTRLENPIXELS` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:123`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:325-326`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:665-666`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:665-666`
pub struct CgRFontstrlenpixels;

impl OutboundSysCall for CgRFontstrlenpixels {
    type Import = SpCgameImport;
    type Args = CgRFontstrlenpixelsArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_FONTSTRLENPIXELS;
}

impl EncodeSysCall for CgRFontstrlenpixels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.text()),
            args.i_font_index() as isize,
            pass_float(args.scale()),
        ])
    }
}

impl DecodeSysCallReturn for CgRFontstrlenpixels {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
