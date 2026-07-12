use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_CVAR_CREATE` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/code/ui/ui_public.h:160`
/// Args source: `oracle/code/client/cl_ui.cpp:137` and `oracle/code/client/cl_ui.cpp:375-377`
/// Output source: `oracle/code/client/cl_ui.cpp:375-377`
/// Transport/switch source: `oracle/code/client/cl_ui.cpp:375-377`
pub struct UiCvarCreate;

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

impl OutboundSysCall for UiCvarCreate {
    type Import = SpUiImport;
    type Args = UiCvarCreateArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_CREATE;
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
