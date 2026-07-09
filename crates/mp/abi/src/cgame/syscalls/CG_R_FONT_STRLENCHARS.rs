use core::ffi::c_int;
use std::ffi::CString;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_FONT_STRLENCHARS`.
#[derive(Debug)]
pub struct CgRFontStrlencharsArgs {
    text: CString,
}

impl CgRFontStrlencharsArgs {
    pub fn new(text: CString) -> Self {
        Self { text }
    }

    pub fn text(&self) -> &CString {
        &self.text
    }
}

/// `CG_R_FONT_STRLENCHARS` MP cgame imports syscall ABI token.
///
/// Source: `oracle/codemp/cgame/cg_public.h:123`
pub struct CgRFontStrlenchars;

impl OutboundSysCall for CgRFontStrlenchars {
    type Import = MpCgameImport;
    type Args = CgRFontStrlencharsArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_FONT_STRLENCHARS;
}

impl EncodeSysCall for CgRFontStrlenchars {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.text().as_ptr())])
    }
}

impl DecodeSysCallReturn for CgRFontStrlenchars {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
