use core::ffi::c_char;

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_CVAR_VARIABLEVALUE`.
///
/// Raven wrapper: `temp = syscall( UI_CVAR_VARIABLEVALUE, var_name );`
/// Raven transport: `FloatAsInt( Cvar_VariableValue( (const char *)VMA(1) ) )`.
///
/// Args source: `oracle/codemp/ui/ui_syscalls.c:45-48`
/// Args source: `oracle/codemp/ui/ui_local.h:921`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:880-881`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCvarVariablevalueArgs {
    var_name: *const c_char,
}

impl UiCvarVariablevalueArgs {
    pub const fn new(var_name: *const c_char) -> Self {
        Self { var_name }
    }

    pub const fn var_name(&self) -> *const c_char {
        self.var_name
    }
}

/// `UI_CVAR_VARIABLEVALUE` MP UI imports syscall ABI token.
///
/// Raven returns the float bits through the integer syscall word.
///
/// Enum value source: `oracle/codemp/ui/ui_public.h:22`
/// Args source: `oracle/codemp/ui/ui_syscalls.c:45-48`
/// Output source: `oracle/codemp/ui/ui_syscalls.c:45-48`
/// Output source: `oracle/codemp/client/cl_ui.cpp:880-881`
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:880-881`
pub struct UiCvarVariablevalue;

impl OutboundSysCall for UiCvarVariablevalue {
    type Import = MpUiImport;
    type Args = UiCvarVariablevalueArgs;
    type Output = f32;

    const IMPORT: MpUiImport = MpUiImport::UI_CVAR_VARIABLEVALUE;
}

impl EncodeSysCall for UiCvarVariablevalue {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.var_name())])
    }
}

impl DecodeSysCallReturn for UiCvarVariablevalue {
    fn decode_return(word: isize) -> Self::Output {
        f32::from_bits(word as i32 as u32)
    }
}
