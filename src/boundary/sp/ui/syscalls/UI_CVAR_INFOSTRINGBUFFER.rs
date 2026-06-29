use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_CVAR_INFOSTRINGBUFFER` SP UI imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:161`
/// Args source: `oracle/oracle/code/client/cl_ui.cpp:428`
/// Output source: `oracle/oracle/code/client/cl_ui.cpp:428`
/// Transport/switch source: `oracle/oracle/code/client/cl_ui.cpp:428`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCvarInfostringbufferArgs {
    bit: c_int,
    buffer: *mut c_char,
    bufsize: c_int,
}

impl UiCvarInfostringbufferArgs {
    pub const fn new(bit: c_int, buffer: *mut c_char, bufsize: c_int) -> Self {
        Self {
            bit,
            buffer,
            bufsize,
        }
    }

    pub const fn bit(&self) -> c_int {
        self.bit
    }

    pub const fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub const fn bufsize(&self) -> c_int {
        self.bufsize
    }
}

pub struct UiCvarInfostringbuffer;

impl OutboundSysCall for UiCvarInfostringbuffer {
    type Import = SpUiImport;
    type Args = UiCvarInfostringbufferArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_CVAR_INFOSTRINGBUFFER;
}

impl EncodeSysCall for UiCvarInfostringbuffer {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.bit() as isize,
            ptr_to_word(args.buffer()),
            args.bufsize() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiCvarInfostringbuffer {
    fn decode_return(_word: isize) -> Self::Output {}
}
