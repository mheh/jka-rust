use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_R_SET_LIGHT_STYLE`.
///
/// Raven wrapper: `void trap_R_SetLightStyle(int style, int color)`.
/// Raven transport: `re.SetLightStyle(args[1], args[2]);`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:504-506`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:805-807`
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

/// `CG_R_SET_LIGHT_STYLE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:181`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:504-506`
/// Output source: `oracle/code/client/cl_cgame.cpp:805-807`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:805-807`
pub struct CgRSetLightStyle;

impl OutboundSysCall for CgRSetLightStyle {
    type Import = SpCgameImport;
    type Args = CgRSetLightStyleArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_R_SET_LIGHT_STYLE;
}

impl EncodeSysCall for CgRSetLightStyle {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.style() as isize, args.color() as isize])
    }
}

impl DecodeSysCallReturn for CgRSetLightStyle {
    fn decode_return(_word: isize) -> Self::Output {}
}
