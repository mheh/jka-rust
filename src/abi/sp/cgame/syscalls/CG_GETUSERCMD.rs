use core::ffi::{c_int, c_void};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_GETUSERCMD`.
///
/// Raven wrapper: `return syscall( CG_GETUSERCMD, cmdNumber, ucmd );`
/// Raven transport: `return CL_GetUserCmd(args[1], (usercmd_s *)VMA(2));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:471-472`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:768-769`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgGetusercmdArgs {
    cmd_number: c_int,
    ucmd: *mut c_void,
}

impl CgGetusercmdArgs {
    pub const fn new(cmd_number: c_int, ucmd: *mut c_void) -> Self {
        Self { cmd_number, ucmd }
    }
}

/// `CG_GETUSERCMD` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:159`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:471-472`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:768-769`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:768-769`
pub struct CgGetusercmd;

impl OutboundSysCall for CgGetusercmd {
    type Import = SpCgameImport;
    type Args = CgGetusercmdArgs;
    type Output = qboolean;

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETUSERCMD;
}

impl EncodeSysCall for CgGetusercmd {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.cmd_number as isize, ptr_to_word(args.ucmd)])
    }
}

impl DecodeSysCallReturn for CgGetusercmd {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
