use core::ffi::c_char;

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for the `UI_ERROR` syscall.
///
/// ABI: `void trap_Error(const char *fmt)`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:25`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:691`
#[derive(Debug)]
pub struct CgErrorArgs {
    /// NUL-terminated message string passed to `Com_Error`.
    message: *const c_char,
}

impl CgErrorArgs {
    /// Construct raw `trap_Error` syscall args.
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

/// `UI_ERROR` MP cgame imports syscall boundary token.
///
/// Source: `oracle/oracle/codemp/ui/ui_public.h:58`
pub struct CgError;

impl OutboundSysCall for CgError {
    type Import = MpUiImport;
    type Args = CgErrorArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_ERROR;
}

impl EncodeSysCall for CgError {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.message())])
    }
}

impl DecodeSysCallReturn for CgError {
    /// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:693`
    /// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:691`
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
