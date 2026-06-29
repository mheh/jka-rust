use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_ERROR`.
///
/// Raven wrapper: `syscall( CG_ERROR, fmt );`
/// Raven transport: `Com_Error( ERR_DROP, S_COLOR_RED"%s", VMA(1) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:50-52`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:440-442`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgErrorArgs {
    message: *const c_char,
}

impl CgErrorArgs {
    /// # Safety
    /// `message` must point to a valid NUL-terminated C string.
    pub const unsafe fn new(message: *const c_char) -> Self {
        Self { message }
    }
}

/// `CG_ERROR` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:62`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:50-52`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:440-442`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:440-442`
pub struct CgError;

impl OutboundSysCall for CgError {
    type Import = SpCgameImport;
    type Args = CgErrorArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_ERROR;
}

impl EncodeSysCall for CgError {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.message)])
    }
}

impl DecodeSysCallReturn for CgError {
    fn decode_return(_word: isize) -> Self::Output {}
}
