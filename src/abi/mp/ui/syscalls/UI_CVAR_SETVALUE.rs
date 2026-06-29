use core::ffi::c_char;

use super::super::MpUiImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::syscalls::pass_float;

/// Arguments for `UI_CVAR_SETVALUE`.
///
/// Raven wrapper: `syscall( UI_CVAR_SETVALUE, var_name, PASSFLOAT( value ) );`
/// Raven transport: `Cvar_SetValue( (const char *)VMA(1), VMF(2) );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:55-56`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:923`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:887-889`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiCvarSetvalueArgs {
    var_name: *const c_char,
    value: f32,
}

impl UiCvarSetvalueArgs {
    pub const fn new(var_name: *const c_char, value: f32) -> Self {
        Self { var_name, value }
    }

    pub const fn var_name(&self) -> *const c_char {
        self.var_name
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

/// `UI_CVAR_SETVALUE` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:24`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:55-56`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:887-889`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:887-889`
pub struct UiCvarSetvalue;

impl OutboundSysCall for UiCvarSetvalue {
    type Import = MpUiImport;
    type Args = UiCvarSetvalueArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_CVAR_SETVALUE;
}

impl EncodeSysCall for UiCvarSetvalue {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.var_name()), pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiCvarSetvalue {
    fn decode_return(_word: isize) -> Self::Output {}
}
