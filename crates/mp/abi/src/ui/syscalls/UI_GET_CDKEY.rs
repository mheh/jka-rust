use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_GET_CDKEY`.
///
/// Raven wrapper: `syscall( UI_GET_CDKEY, buf, buflen );`
/// Raven transport: `CLUI_GetCDKey( (char *)VMA(1), args[2] ); return 0;`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:348-349`
/// Args source: `oracle/codemp/ui/ui_local.h:986`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1123-1125`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiGetCdkeyArgs {
    buf: *mut c_char,
    buflen: c_int,
}

impl UiGetCdkeyArgs {
    pub const fn new(buf: *mut c_char, buflen: c_int) -> Self {
        Self { buf, buflen }
    }

    pub const fn buf(&self) -> *mut c_char {
        self.buf
    }

    pub const fn buflen(&self) -> c_int {
        self.buflen
    }
}

/// `UI_GET_CDKEY` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:72`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:348-349`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1123-1125`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1123-1125`
pub struct UiGetCdkey;

impl OutboundSysCall for UiGetCdkey {
    type Import = MpUiImport;
    type Args = UiGetCdkeyArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_GET_CDKEY;
}

impl EncodeSysCall for UiGetCdkey {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.buf()), args.buflen() as isize])
    }
}

impl DecodeSysCallReturn for UiGetCdkey {
    fn decode_return(_word: isize) -> Self::Output {}
}
