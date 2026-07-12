use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// Arguments for `CG_R_FONT_STRHEIGHTPIXELS`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgRFontStrheightpixelsArgs {
    font_index: c_int,
    scale: f32,
}

impl CgRFontStrheightpixelsArgs {
    pub const fn new(font_index: c_int, scale: f32) -> Self {
        Self { font_index, scale }
    }

    pub const fn font_index(&self) -> c_int {
        self.font_index
    }

    pub const fn scale(&self) -> f32 {
        self.scale
    }
}

/// `CG_R_FONT_STRHEIGHTPIXELS` MP cgame imports syscall ABI token.
///
/// Source: `oracle/codemp/cgame/cg_public.h:124`
pub struct CgRFontStrheightpixels;

impl OutboundSysCall for CgRFontStrheightpixels {
    type Import = MpCgameImport;
    type Args = CgRFontStrheightpixelsArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_FONT_STRHEIGHTPIXELS;
}

impl EncodeSysCall for CgRFontStrheightpixels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.font_index() as isize, pass_float(args.scale())])
    }
}

impl DecodeSysCallReturn for CgRFontStrheightpixels {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
