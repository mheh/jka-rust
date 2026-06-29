use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_GETCLIPBOARDDATA`.
///
/// Raven wrapper: `syscall( UI_GETCLIPBOARDDATA, buf, bufsize );`
/// Raven transport: `GetClipboardData((char *)VMA(1), args[2]); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:254-255`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1040-1042`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiGetclipboarddataArgs {
    buf: *mut c_char,
    bufsize: c_int,
}

impl UiGetclipboarddataArgs {
    pub const fn new(buf: *mut c_char, bufsize: c_int) -> Self {
        Self { buf, bufsize }
    }

    pub const fn buf(&self) -> *mut c_char {
        self.buf
    }

    pub const fn bufsize(&self) -> c_int {
        self.bufsize
    }
}

/// `UI_GETCLIPBOARDDATA` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:61`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:52-62`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:254-255`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1040-1042`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1040-1042`
pub struct UiGetclipboarddata;

impl OutboundSysCall for UiGetclipboarddata {
    type Import = MpUiImport;
    type Args = UiGetclipboarddataArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_GETCLIPBOARDDATA;
}

impl EncodeSysCall for UiGetclipboarddata {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.buf()), args.bufsize() as isize])
    }
}

impl DecodeSysCallReturn for UiGetclipboarddata {
    fn decode_return(_word: isize) -> Self::Output {}
}
