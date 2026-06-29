use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_PRINT`.
///
/// Raven wrapper: `syscall( CG_PRINT, fmt );`
/// Raven transport: `Com_Printf( "%s", VMA(1) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:46-48`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:437-439`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgPrintArgs {
    message: *const c_char,
}

impl CgPrintArgs {
    /// # Safety
    /// `message` must point to a valid NUL-terminated C string.
    pub const unsafe fn new(message: *const c_char) -> Self {
        Self { message }
    }
}

/// `CG_PRINT` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:61`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:46-48`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:437-439`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:437-439`
pub struct CgPrint;

impl OutboundSysCall for CgPrint {
    type Import = SpCgameImport;
    type Args = CgPrintArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_PRINT;
}

impl EncodeSysCall for CgPrint {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.message)])
    }
}

impl DecodeSysCallReturn for CgPrint {
    fn decode_return(_word: isize) -> Self::Output {}
}
