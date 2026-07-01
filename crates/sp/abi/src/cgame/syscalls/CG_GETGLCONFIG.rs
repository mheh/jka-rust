use core::ffi::c_void;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_GETGLCONFIG`.
///
/// Raven cgame calls `syscall( CG_GETGLCONFIG, glconfig )`; the SP client switch
/// decodes that argument as `(glconfig_t *)VMA(1)` and fills it via
/// `CL_GetGlconfig`.
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:442-443`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:749-751`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgGetglconfigArgs {
    glconfig: *mut c_void,
}

impl CgGetglconfigArgs {
    pub const unsafe fn new(glconfig: *mut c_void) -> Self {
        Self { glconfig }
    }

    pub const fn glconfig(&self) -> *mut c_void {
        self.glconfig
    }
}

/// `CG_GETGLCONFIG` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:150`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:442-443`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:749-751`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:749-751`
pub struct CgGetglconfig;

impl OutboundSysCall for CgGetglconfig {
    type Import = SpCgameImport;
    type Args = CgGetglconfigArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETGLCONFIG;
}

impl EncodeSysCall for CgGetglconfig {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.glconfig())])
    }
}

impl DecodeSysCallReturn for CgGetglconfig {
    fn decode_return(_word: isize) -> Self::Output {}
}
