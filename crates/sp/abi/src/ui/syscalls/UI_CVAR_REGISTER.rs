use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::vmCvar_t;

/// `UI_CVAR_REGISTER` SP UI imports syscall ABI token.
///
/// Raven: 50
/// Enum value source: `oracle/code/ui/ui_public.h:202`
/// Args source: `oracle/code/client/client.h:340` and `oracle/code/client/cl_ui.cpp:375-377`
/// Output source: `oracle/code/client/cl_ui.cpp:375-377`
/// Transport/switch source: `oracle/code/client/cl_ui.cpp:375-377`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCvarRegisterArgs {
    cvar: *mut vmCvar_t,
    var_name: *const c_char,
    default_value: *const c_char,
    flags: c_int,
}

impl UiCvarRegisterArgs {
    /// `vmCvar_t` is passed for in/out mirror updates by the engine.
    pub const unsafe fn new(
        cvar: *mut vmCvar_t,
        var_name: *const c_char,
        default_value: *const c_char,
        flags: c_int,
    ) -> Self {
        Self {
            cvar,
            var_name,
            default_value,
            flags,
        }
    }

    pub const fn cvar(&self) -> *mut vmCvar_t {
        self.cvar
    }

    pub const fn var_name(&self) -> *const c_char {
        self.var_name
    }

    pub const fn default_value(&self) -> *const c_char {
        self.default_value
    }

    pub const fn flags(&self) -> c_int {
        self.flags
    }
}

pub struct UiCvarRegister;

impl OutboundSysCall for UiCvarRegister {
    type Import = SpUiImport;
    type Args = UiCvarRegisterArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_REGISTER;
}

impl EncodeSysCall for UiCvarRegister {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.cvar()),
            ptr_to_word(args.var_name()),
            ptr_to_word(args.default_value()),
            args.flags() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiCvarRegister {
    fn decode_return(_word: isize) -> Self::Output {}
}
