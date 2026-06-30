use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_GETCURRENTCMDNUMBER`.
///
/// Raven's `trap_GetCurrentCmdNumber` forwards only the syscall token, so this
/// call has no transport payload.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:486`
/// Transport source: `oracle/oracle/codemp/cgame/cg_syscalls.c:487`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:969`
#[derive(Debug, Default)]
pub struct CgGetcurrentcmdnumberArgs;

impl CgGetcurrentcmdnumberArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_GETCURRENTCMDNUMBER` MP cgame imports syscall ABI token.
///
/// C signature: `int trap_GetCurrentCmdNumber(void)`.
/// Raven transport: `return syscall( CG_GETCURRENTCMDNUMBER );`
/// Raven switch: `return CL_GetCurrentCmdNumber();`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:185`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:486-487`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:486-487`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:970`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:969-970`
pub struct CgGetcurrentcmdnumber;

impl OutboundSysCall for CgGetcurrentcmdnumber {
    type Import = MpCgameImport;
    type Args = CgGetcurrentcmdnumberArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_GETCURRENTCMDNUMBER;
}

impl EncodeSysCall for CgGetcurrentcmdnumber {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgGetcurrentcmdnumber {
    // `CL_GetCurrentCmdNumber` returns an `int`; the engine's return word is that value.
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
