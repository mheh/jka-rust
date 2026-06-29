use core::ffi::c_char;

use super::super::SpUiImport;
use crate::abi::generic::{
    DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport, ptr_to_word,
};

/// `UI_ERROR` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:152`
/// Args source: `oracle/oracle/code/client/cl_ui.cpp:371`
/// Output source: `oracle/oracle/code/client/cl_ui.cpp:371`
/// Transport/switch source: `oracle/oracle/code/client/cl_ui.cpp:371`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiErrorArgs {
    message: *const c_char,
}

impl UiErrorArgs {
    /// # Safety
    /// `message` must point to a valid NUL-terminated C string for the syscall.
    pub const unsafe fn new(message: *const c_char) -> Self {
        Self { message }
    }

    pub const fn message(&self) -> *const c_char {
        self.message
    }
}

pub struct UiError;

impl OutboundSysCall for UiError {
    type Import = SpUiImport;
    type Args = UiErrorArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_ERROR;
}

impl EncodeSysCall for UiError {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.message())])
    }
}

impl DecodeSysCallReturn for UiError {
    fn decode_return(_word: isize) -> Self::Output {}
}
