use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_KEY_KEYNUMTOSTRINGBUF`.
///
/// Raven wrapper: `syscall( UI_KEY_KEYNUMTOSTRINGBUF, keynum, buf, buflen );`
/// Raven transport: `Key_KeynumToStringBuf(args[1], (char *)VMA(2), args[3]); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:218-219`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1007-1009`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiKeyKeynumtostringbufArgs {
    keynum: c_int,
    buf: *mut c_char,
    buflen: c_int,
}

impl UiKeyKeynumtostringbufArgs {
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

/// `UI_KEY_KEYNUMTOSTRINGBUF` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:52`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:52-62`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:218-219`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1007-1009`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1007-1009`
pub struct UiKeyKeynumtostringbuf;

impl OutboundSysCall for UiKeyKeynumtostringbuf {
    type Import = MpUiImport;
    type Args = UiKeyKeynumtostringbufArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_KEYNUMTOSTRINGBUF;
}

impl EncodeSysCall for UiKeyKeynumtostringbuf {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.keynum() as isize,
            ptr_to_word(args.buf()),
            args.buflen() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiKeyKeynumtostringbuf {
    fn decode_return(_word: isize) -> Self::Output {}
}
