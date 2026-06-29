use core::ffi::c_char;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_PRINT`.
///
/// Raven wrapper: `void trap_Print( const char *fmt )`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:21`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:688`
#[derive(Debug)]
pub struct CgPrintArgs {
    message: *const c_char,
}

impl CgPrintArgs {
    /// Construct raw `trap_Print` syscall args.
    ///
    /// # Safety
    /// `message` must point to a valid NUL-terminated C string for the duration
    /// of the syscall.
    pub const unsafe fn new(message: *const c_char) -> Self {
        Self { message }
    }

    pub const fn message(&self) -> *const c_char {
        self.message
    }
}

/// `CG_PRINT` MP cgame imports syscall ABI token.
///
/// Raven wrapper: `syscall( CG_PRINT, fmt );`
/// Raven transport: `Com_Printf( "%s", VMA(1) );`
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:57`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:21`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:690`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:688`
pub struct CgPrint;

impl OutboundSysCall for CgPrint {
    type Import = MpCgameImport;
    type Args = CgPrintArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_PRINT;
}

impl EncodeSysCall for CgPrint {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.message())])
    }
}

impl DecodeSysCallReturn for CgPrint {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
