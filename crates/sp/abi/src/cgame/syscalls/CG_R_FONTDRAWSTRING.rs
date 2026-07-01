use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `CG_R_FONTDRAWSTRING`.
///
/// Raven wrapper: `syscall (CG_R_FONTDRAWSTRING, ox, oy, text, rgba, setIndex, iMaxPixelWidth, PASSFLOAT(scale) );`
/// Raven transport: `re.Font_DrawString(args[1], args[2], (const char *) VMA(3), (float*) VMA(4), args[5], args[6], VMF(7));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:352-353`
/// Args source: `oracle/oracle/code/cgame/cg_local.h:1025`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:671-673`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRFontdrawstringArgs {
    ox: c_int,
    oy: c_int,
    text: *const c_char,
    rgba: *const f32,
    set_index: c_int,
    max_pixel_width: c_int,
    scale: f32,
}

impl CgRFontdrawstringArgs {
    pub const fn new(
        ox: c_int,
        oy: c_int,
        text: *const c_char,
        rgba: *const f32,
        set_index: c_int,
        max_pixel_width: c_int,
        scale: f32,
    ) -> Self {
        Self {
            ox,
            oy,
            text,
            rgba,
            set_index,
            max_pixel_width,
            scale,
        }
    }

    pub const fn ox(&self) -> c_int {
        self.ox
    }

    pub const fn oy(&self) -> c_int {
        self.oy
    }

    pub const fn text(&self) -> *const c_char {
        self.text
    }

    pub const fn rgba(&self) -> *const f32 {
        self.rgba
    }

    pub const fn set_index(&self) -> c_int {
        self.set_index
    }

    pub const fn max_pixel_width(&self) -> c_int {
        self.max_pixel_width
    }

    pub const fn scale(&self) -> f32 {
        self.scale
    }
}

/// `CG_R_FONTDRAWSTRING` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:126`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:352-353`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:671-673`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:671-673`
pub struct CgRFontdrawstring;

impl OutboundSysCall for CgRFontdrawstring {
    type Import = SpCgameImport;
    type Args = CgRFontdrawstringArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_FONTDRAWSTRING;
}

impl EncodeSysCall for CgRFontdrawstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.ox() as isize,
            args.oy() as isize,
            ptr_to_word(args.text()),
            ptr_to_word(args.rgba()),
            args.set_index() as isize,
            args.max_pixel_width() as isize,
            pass_float(args.scale()),
        ])
    }
}

impl DecodeSysCallReturn for CgRFontdrawstring {
    fn decode_return(_word: isize) -> Self::Output {}
}
