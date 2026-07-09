use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_R_SET_LIGHT_STYLE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRSetLightStyleArgs {
    style: c_int,
    color: c_int,
}

impl CgRSetLightStyleArgs {
    pub const fn new(style: c_int, color: c_int) -> Self {
        Self { style, color }
    }

    pub const fn style(&self) -> c_int {
        self.style
    }

    pub const fn color(&self) -> c_int {
        self.color
    }
}

/// `CG_R_SET_LIGHT_STYLE` MP cgame imports syscall ABI token.
///
/// Source: `oracle/codemp/cgame/cg_public.h:169`
pub struct CgRSetLightStyle;

impl OutboundSysCall for CgRSetLightStyle {
    type Import = MpCgameImport;
    type Args = CgRSetLightStyleArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_R_SET_LIGHT_STYLE;
}

impl EncodeSysCall for CgRSetLightStyle {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.style() as isize, args.color() as isize])
    }
}

impl DecodeSysCallReturn for CgRSetLightStyle {
    fn decode_return(_word: isize) -> Self::Output {}
}
