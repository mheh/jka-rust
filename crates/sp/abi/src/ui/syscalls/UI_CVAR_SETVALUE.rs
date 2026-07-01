use core::ffi::c_char;

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use abi_transport::pass_float;

/// `UI_CVAR_SETVALUE` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:158`
/// Args source: `oracle/oracle/code/client/cl_ui.cpp` is missing for this SP call;
/// `oracle/oracle/codemp/ui/ui_syscalls.c:55-56` provides Raven's canonical wrapper form.
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:55-56` and
/// `oracle/oracle/codemp/client/cl_ui.cpp:887-889`.
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:887-889`.
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

pub struct UiCvarSetvalue;

impl OutboundSysCall for UiCvarSetvalue {
    type Import = SpUiImport;
    type Args = UiCvarSetvalueArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_SETVALUE;
}

impl EncodeSysCall for UiCvarSetvalue {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.var_name()), pass_float(args.value())])
    }
}

impl DecodeSysCallReturn for UiCvarSetvalue {
    fn decode_return(_word: isize) -> Self::Output {}
}
