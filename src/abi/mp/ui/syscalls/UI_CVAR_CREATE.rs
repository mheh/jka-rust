use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_CVAR_CREATE`.
///
/// Raven wrapper: `syscall( UI_CVAR_CREATE, var_name, var_value, flags );`
/// Raven transport: `Cvar_Get( (const char *)VMA(1), (const char *)VMA(2), args[3] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:63-64`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:925`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:895-897`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCvarCreateArgs {
    var_name: *const c_char,
    var_value: *const c_char,
    flags: c_int,
}

impl UiCvarCreateArgs {
    pub const fn new(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> Self {
        Self {
            var_name,
            var_value,
            flags,
        }
    }

    pub const fn var_name(&self) -> *const c_char {
        self.var_name
    }

    pub const fn var_value(&self) -> *const c_char {
        self.var_value
    }

    pub const fn flags(&self) -> c_int {
        self.flags
    }
}

/// `UI_CVAR_CREATE` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:26`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:63-64`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:63-64`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:895-897`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:895-897`
pub struct UiCvarCreate;

impl OutboundSysCall for UiCvarCreate {
    type Import = MpUiImport;
    type Args = UiCvarCreateArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_CVAR_CREATE;
}

impl EncodeSysCall for UiCvarCreate {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.var_name()),
            ptr_to_word(args.var_value()),
            args.flags() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiCvarCreate {
    fn decode_return(_word: isize) -> Self::Output {}
}
