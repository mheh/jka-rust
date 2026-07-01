use core::ffi::{c_char, c_int};

use super::super::MpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `UI_CVAR_INFOSTRINGBUFFER`.
///
/// Raven wrapper: `syscall( UI_CVAR_INFOSTRINGBUFFER, bit, buffer, bufsize );`
/// Raven transport: `Cvar_InfoStringBuffer( args[1], (char *)VMA(2), args[3] );`
///
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:67-68`
/// Args source: `oracle/oracle/codemp/ui/ui_local.h:926`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:899-901`
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

/// `UI_CVAR_INFOSTRINGBUFFER` MP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/ui/ui_public.h:27`
/// Args source: `oracle/oracle/codemp/ui/ui_syscalls.c:67-68`
/// Output source: `oracle/oracle/codemp/ui/ui_syscalls.c:67-68`
/// Output source: `oracle/oracle/codemp/client/cl_ui.cpp:899-901`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_ui.cpp:899-901`
pub struct UiCvarInfostringbuffer;

impl OutboundSysCall for UiCvarInfostringbuffer {
    type Import = MpUiImport;
    type Args = UiCvarInfostringbufferArgs;
    type Output = ();

    const IMPORT: MpUiImport = MpUiImport::UI_CVAR_INFOSTRINGBUFFER;
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
