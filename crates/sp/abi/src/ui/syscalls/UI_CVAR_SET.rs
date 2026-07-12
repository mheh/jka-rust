use core::ffi::c_char;

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_CVAR_SET` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/code/ui/ui_public.h:155`
/// Args source: `oracle/code/client/cl_ui.cpp` is missing for this SP call;
/// `oracle/codemp/ui/ui_syscalls.c:41-42` provides Raven's canonical wrapper form.
/// Output source: `oracle/codemp/ui/ui_syscalls.c:41-42` and
/// `oracle/codemp/client/cl_ui.cpp:876-878`.
/// Transport/switch source: `oracle/codemp/client/cl_ui.cpp:876-878`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCvarSetArgs {
    var_name: *const c_char,
    value: *const c_char,
}

impl UiCvarSetArgs {
    /// Construct raw `trap_Cvar_Set( var_name, value )` payload.
    pub const unsafe fn new(var_name: *const c_char, value: *const c_char) -> Self {
        Self { var_name, value }
    }

    pub const fn var_name(&self) -> *const c_char {
        self.var_name
    }

    pub const fn value(&self) -> *const c_char {
        self.value
    }
}

pub struct UiCvarSet;

impl OutboundSysCall for UiCvarSet {
    type Import = SpUiImport;
    type Args = UiCvarSetArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_SET;
}

impl EncodeSysCall for UiCvarSet {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.var_name()), ptr_to_word(args.value())])
    }
}

impl DecodeSysCallReturn for UiCvarSet {
    fn decode_return(_word: isize) -> Self::Output {}
}
