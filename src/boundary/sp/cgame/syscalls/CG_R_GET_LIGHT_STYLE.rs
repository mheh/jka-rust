use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_GET_LIGHT_STYLE`.
///
/// Raven wrapper: `void trap_R_GetLightStyle(int style, color4ub_t color)`.
/// Raven transport: `re.GetLightStyle(args[1], (byte *) VMA(2));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:499-501`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:802-804`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRGetLightStyleArgs {
    style: c_int,
    color: *mut u8,
}

impl CgRGetLightStyleArgs {
    /// `color` should point to a 4-byte RGBA buffer.
    pub const fn new(style: c_int, color: *mut u8) -> Self {
        Self { style, color }
    }

    pub const fn style(&self) -> c_int {
        self.style
    }

    pub const fn color(&self) -> *mut u8 {
        self.color
    }
}

/// `CG_R_GET_LIGHT_STYLE` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:180`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:499-501`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:802-804`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:802-804`
pub struct CgRGetLightStyle;

impl OutboundSysCall for CgRGetLightStyle {
    type Import = SpCgameImport;
    type Args = CgRGetLightStyleArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_GET_LIGHT_STYLE;
}

impl EncodeSysCall for CgRGetLightStyle {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.style() as isize, ptr_to_word(args.color())])
    }
}

impl DecodeSysCallReturn for CgRGetLightStyle {
    fn decode_return(_word: isize) -> Self::Output {}
}
