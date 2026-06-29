use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_GETCURRENTCMDNUMBER`.
///
/// Raven wrapper: `return syscall( CG_GETCURRENTCMDNUMBER );`
/// Raven transport: `return CL_GetCurrentCmdNumber();`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:467-468`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:766-767`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgGetcurrentcmdnumberArgs;

impl CgGetcurrentcmdnumberArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_GETCURRENTCMDNUMBER` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:158`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:467-468`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:766-767`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:766-767`
pub struct CgGetcurrentcmdnumber;

impl OutboundSysCall for CgGetcurrentcmdnumber {
    type Import = SpCgameImport;
    type Args = CgGetcurrentcmdnumberArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_GETCURRENTCMDNUMBER;
}

impl EncodeSysCall for CgGetcurrentcmdnumber {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgGetcurrentcmdnumber {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
