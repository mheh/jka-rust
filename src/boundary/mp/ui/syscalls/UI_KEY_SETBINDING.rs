use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_KEY_SETBINDING`.
///
/// Raven wrapper: `syscall( UI_KEY_SETBINDING, keynum, binding );`
/// Raven transport: `Key_SetBinding(args[1], (const char *)VMA(2)); return 0;`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:226-227`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1015-1017`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiKeySetbindingArgs {
    keynum: c_int,
    binding: *const c_char,
}

impl UiKeySetbindingArgs {
    pub const fn new(keynum: c_int, binding: *const c_char) -> Self {
        Self { keynum, binding }
    }

    pub const fn keynum(&self) -> c_int {
        self.keynum
    }

    pub const fn binding(&self) -> *const c_char {
        self.binding
    }
}

/// `UI_KEY_SETBINDING` MP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:54`
/// Enum comment source: `oracle/oracle/codemp/ui/ui_public.h:52-62`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:226-227`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:1015-1017`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:1015-1017`
pub struct UiKeySetbinding;

impl OutboundSysCall for UiKeySetbinding {
    type Import = MpUiImport;
    type Args = UiKeySetbindingArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_KEY_SETBINDING;
}

impl EncodeSysCall for UiKeySetbinding {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.keynum() as isize, ptr_to_word(args.binding())])
    }
}

impl DecodeSysCallReturn for UiKeySetbinding {
    fn decode_return(_word: isize) -> Self::Output {}
}
