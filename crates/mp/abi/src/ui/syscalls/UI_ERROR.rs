use core::ffi::c_char;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for the `UI_ERROR` syscall.
///
/// ABI: `void trap_Error(const char *fmt)`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:25`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:691`
#[derive(Debug)]
pub struct UiErrorArgs {
    /// NUL-terminated message string passed to `Com_Error`.
    message: *const c_char,
}

impl UiErrorArgs {
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

/// `UI_ERROR` MP UI imports syscall ABI token.
///
/// Source: `oracle/codemp/ui/ui_public.h:58`
pub struct UiError;

impl OutboundSysCall for UiError {
    type Import = MpUiImport;
    type Args = UiErrorArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_ERROR;
}

impl EncodeSysCall for UiError {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.message())])
    }
}

impl DecodeSysCallReturn for UiError {
    /// Output source: `oracle/codemp/client/cl_ui.cpp:693`
    /// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:691`
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
