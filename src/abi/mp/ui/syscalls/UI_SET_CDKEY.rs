use core::ffi::c_char;

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_SET_CDKEY`.
///
/// Raven wrapper: `syscall( UI_SET_CDKEY, buf );`
/// Raven transport: `CLUI_SetCDKey( (char *)VMA(1) ); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:352-353`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:987`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1127-1129`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSetCdkeyArgs {
    buf: *mut c_char,
}

impl UiSetCdkeyArgs {
    pub const fn new(buf: *mut c_char) -> Self {
        Self { buf }
    }

    pub const fn buf(&self) -> *mut c_char {
        self.buf
    }
}

/// `UI_SET_CDKEY` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:73`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:352-353`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1127-1129`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1127-1129`
pub struct UiSetCdkey;

impl OutboundSysCall for UiSetCdkey {
    type Import = MpUiImport;
    type Args = UiSetCdkeyArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_SET_CDKEY;
}

impl EncodeSysCall for UiSetCdkey {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.buf())])
    }
}

impl DecodeSysCallReturn for UiSetCdkey {
    fn decode_return(_word: isize) -> Self::Output {}
}
