use super::super::SpUiImport;
use core::ffi::{c_char, c_int};
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_GETCLIPBOARDDATA` SP UI imports syscall ABI token.
///
/// Source: `oracle/oracle/code/ui/ui_public.h:194`
pub struct UiGetclipboarddata;

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

impl OutboundSysCall for UiGetclipboarddata {
    type Import = SpUiImport;
    /// Raven wrapper: `syscall( UI_GETCLIPBOARDDATA, buf, bufsize );`
    ///
    /// Args source: `oracle/oracle/code/ui/ui_syscalls.cpp:147-149`
    /// Output source: `oracle/oracle/code/ui/ui_syscalls.cpp:147-149`
    ///
    /// Transport/switch source: `oracle/oracle/code/ui/ui_syscalls.cpp:147-149`
    type Args = UiGetclipboarddataArgs;
    /// This call writes into the caller-provided buffer and does not return a value.
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_GETCLIPBOARDDATA;
}

impl EncodeSysCall for UiGetclipboarddata {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.buf()), args.bufsize() as isize])
    }
}

impl DecodeSysCallReturn for UiGetclipboarddata {
    fn decode_return(_word: isize) -> Self::Output {}
}
