use core::ffi::c_char;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_PRINT`.
///
/// Raven wrapper: `void trap_Print( const char *fmt )`.
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:21`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:688`
#[derive(Debug)]
pub struct UiPrintArgs {
    message: *const c_char,
}

impl UiPrintArgs {
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

/// `UI_PRINT` MP UI imports syscall ABI token.
///
/// Raven wrapper: `syscall( UI_PRINT, fmt );`
/// Raven transport: `Com_Printf( "%s", VMA(1) );`
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:57`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:21`
/// Output source: `oracle/codemp/client/cl_ui.cpp:690`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:688`
pub struct UiPrint;

impl OutboundSysCall for UiPrint {
    type Import = MpUiImport;
    type Args = UiPrintArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_PRINT;
}

impl EncodeSysCall for UiPrint {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.message())])
    }
}

impl DecodeSysCallReturn for UiPrint {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
