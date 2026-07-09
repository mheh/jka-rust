use core::ffi::c_char;

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_CVAR_VARIABLEVALUE` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/code/ui/ui_public.h:156`
/// Args source: `oracle/code/client/cl_ui.cpp:405-406`.
/// Output source: `oracle/code/client/cl_ui.cpp:405-406`.
/// Transport/switch source: `oracle/code/client/cl_ui.cpp:405-406`.
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

pub struct UiCvarVariablevalue;

impl OutboundSysCall for UiCvarVariablevalue {
    type Import = SpUiImport;
    type Args = UiCvarVariablevalueArgs;
    type Output = f32;

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_VARIABLEVALUE;
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
