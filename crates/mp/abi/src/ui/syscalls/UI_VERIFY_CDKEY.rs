use core::ffi::c_char;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::qboolean;

/// Arguments for `UI_VERIFY_CDKEY`.
///
/// Raven wrapper: `return syscall( UI_VERIFY_CDKEY, key, chksum);`
/// Raven transport: `return CL_CDKeyValidate((const char *)VMA(1), (const char *)VMA(2));`
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:356-357`
/// Args source: `oracle/codemp/ui/ui_local.h:988`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1206-1207`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiVerifyCdkeyArgs {
    key: *const c_char,
    chksum: *const c_char,
}

impl UiVerifyCdkeyArgs {
    pub const fn new(key: *const c_char, chksum: *const c_char) -> Self {
        Self { key, chksum }
    }

    pub const fn key(&self) -> *const c_char {
        self.key
    }

    pub const fn chksum(&self) -> *const c_char {
        self.chksum
    }
}

/// `UI_VERIFY_CDKEY` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:74`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:356-357`
/// Output source: `oracle/codemp/ui/ui_local.h:988`
/// Output source: `oracle/codemp/client/cl_ui.cpp:1206-1207`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:1206-1207`
pub struct UiVerifyCdkey;

impl OutboundSysCall for UiVerifyCdkey {
    type Import = MpUiImport;
    type Args = UiVerifyCdkeyArgs;
    type Output = qboolean;

    const IMPORT: MpUiImport = MpUiImport::UI_VERIFY_CDKEY;
}

impl EncodeSysCall for UiVerifyCdkey {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.key()), ptr_to_word(args.chksum())])
    }
}

impl DecodeSysCallReturn for UiVerifyCdkey {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
