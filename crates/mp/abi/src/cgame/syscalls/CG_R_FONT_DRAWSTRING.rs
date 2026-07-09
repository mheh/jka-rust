use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `CG_R_FONT_DRAWSTRING`.
#[derive(Debug)]
pub struct CgRFontDrawstringArgs {
    ox: c_int,
    oy: c_int,
    text: CString,
    rgba: *const f32,
    set_index: c_int,
    i_char_limit: c_int,
    scale: f32,
}

impl CgRFontDrawstringArgs {
    /// # Safety
    /// `rgba` must point to at least four readable floats for the duration of the syscall.
    pub fn new(
        ox: c_int,
        oy: c_int,
        text: CString,
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

    pub const fn ox(&self) -> c_int {
        self.ox
    }

    pub const fn oy(&self) -> c_int {
        self.oy
    }

    pub fn text(&self) -> &CString {
        &self.text
    }

    pub const fn rgba(&self) -> *const f32 {
        self.rgba
    }

    pub const fn set_index(&self) -> c_int {
        self.set_index
    }

    pub const fn i_char_limit(&self) -> c_int {
        self.i_char_limit
    }

    pub const fn scale(&self) -> f32 {
        self.scale
    }
}

/// `CG_R_FONT_DRAWSTRING` MP cgame imports syscall ABI token.
///
/// Source: `oracle/codemp/cgame/cg_public.h:125`
pub struct CgRFontDrawstring;

impl OutboundSysCall for CgRFontDrawstring {
    type Import = MpCgameImport;
    type Args = CgRFontDrawstringArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_FONT_DRAWSTRING;
}

impl EncodeSysCall for CgRFontDrawstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.ox() as isize,
            args.oy() as isize,
            ptr_to_word(args.text().as_ptr()),
            ptr_to_word(args.rgba()),
            args.set_index() as isize,
            args.i_char_limit() as isize,
            pass_float(args.scale()),
        ])
    }
}

impl DecodeSysCallReturn for CgRFontDrawstring {
    fn decode_return(_word: isize) -> Self::Output {}
}
