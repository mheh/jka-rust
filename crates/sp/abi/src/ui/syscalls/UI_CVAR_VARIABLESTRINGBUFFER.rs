use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_CVAR_VARIABLESTRINGBUFFER` SP UI imports syscall ABI token.
///
/// Enum source: `oracle/oracle/code/ui/ui_public.h:157`
/// Args source: `oracle/oracle/code/client/cl_ui.cpp:427-429`.
/// Output source: `oracle/oracle/code/client/cl_ui.cpp:427-429`.
/// Transport/switch source: `oracle/oracle/code/client/cl_ui.cpp:427-429`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCvarVariablestringbufferArgs {
    var_name: *const c_char,
    buffer: *mut c_char,
    bufsize: c_int,
}

impl UiCvarVariablestringbufferArgs {
    /// Construct raw `trap_Cvar_VariableStringBuffer( var_name, buffer, bufsize )` payload.
    pub const unsafe fn new(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int) -> Self {
        Self {
            var_name,
            buffer,
            bufsize,
        }
    }

    pub const fn var_name(&self) -> *const c_char {
        self.var_name
    }

    pub const fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub const fn bufsize(&self) -> c_int {
        self.bufsize
    }
}

pub struct UiCvarVariablestringbuffer;

impl OutboundSysCall for UiCvarVariablestringbuffer {
    type Import = SpUiImport;
    type Args = UiCvarVariablestringbufferArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_VARIABLESTRINGBUFFER;
}

impl EncodeSysCall for UiCvarVariablestringbuffer {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.var_name()),
            ptr_to_word(args.buffer()),
            args.bufsize() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiCvarVariablestringbuffer {
    fn decode_return(_word: isize) -> Self::Output {}
}
