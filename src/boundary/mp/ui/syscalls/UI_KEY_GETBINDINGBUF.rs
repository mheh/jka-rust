use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_KEY_GETBINDINGBUF`.
///
/// Raven wrapper: `syscall( UI_KEY_GETBINDINGBUF, keynum, buf, buflen );`
/// Raven transport: `Key_GetBindingBuf(args[1], (char *)VMA(2), args[3]); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:222-223`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1011-1013`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiKeyGetbindingbufArgs {
    keynum: c_int,
    buf: *mut c_char,
    buflen: c_int,
}

impl UiKeyGetbindingbufArgs {
    pub const fn new(keynum: c_int, buf: *mut c_char, buflen: c_int) -> Self {
        Self {
            keynum,
            buf,
            buflen,
        }
    }

    pub const fn keynum(&self) -> c_int {
        self.keynum
    }

    pub const fn buf(&self) -> *mut c_char {
        self.buf
    }

    pub const fn buflen(&self) -> c_int {
        self.buflen
    }
}

/// `UI_KEY_GETBINDINGBUF` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:53`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:52-62`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:222-223`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1011-1013`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1011-1013`
pub struct UiKeyGetbindingbuf;

impl OutboundSysCall for UiKeyGetbindingbuf {
    type Import = MpUiImport;
    type Args = UiKeyGetbindingbufArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_GETBINDINGBUF;
}

impl EncodeSysCall for UiKeyGetbindingbuf {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.keynum() as isize,
            ptr_to_word(args.buf()),
            args.buflen() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiKeyGetbindingbuf {
    fn decode_return(_word: isize) -> Self::Output {}
}
