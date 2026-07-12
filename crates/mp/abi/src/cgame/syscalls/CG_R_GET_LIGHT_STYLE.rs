use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_GET_LIGHT_STYLE`.
///
/// Raven wrapper: `void trap_R_GetLightStyle(int style, color4ub_t color)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRGetLightStyleArgs {
    style: c_int,
    color: *mut [u8; 4],
}

impl CgRGetLightStyleArgs {
    /// # Safety
    /// `color` must point to a writable 4-byte RGBA buffer.
    pub const unsafe fn new(style: c_int, color: *mut [u8; 4]) -> Self {
        Self { style, color }
    }

    pub const fn style(&self) -> c_int {
        self.style
    }

    pub const fn color(&self) -> *mut [u8; 4] {
        self.color
    }
}

/// `CG_R_GET_LIGHT_STYLE` MP cgame imports syscall ABI token.
///
/// Source: `oracle/codemp/cgame/cg_public.h:168`
pub struct CgRGetLightStyle;

impl OutboundSysCall for CgRGetLightStyle {
    type Import = MpCgameImport;
    type Args = CgRGetLightStyleArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_GET_LIGHT_STYLE;
}

impl EncodeSysCall for CgRGetLightStyle {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.style() as isize, ptr_to_word(args.color())])
    }
}

impl DecodeSysCallReturn for CgRGetLightStyle {
    fn decode_return(_word: isize) -> Self::Output {}
}
