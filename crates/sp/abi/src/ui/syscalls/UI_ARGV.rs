use core::ffi::{c_char, c_int};

use super::super::SpUiImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// `UI_ARGV` SP UI imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/ui/ui_public.h:163`
/// Args source: `oracle/oracle/code/qcommon/qcommon.h:291`
/// Output source: `oracle/oracle/code/qcommon/qcommon.h:291`
/// Transport/switch source: `oracle/oracle/code/client/cl_ui.cpp:219` (bound in `uiimport_t`, no dedicated case)
pub struct UiArgv;

#[derive(Debug)]
pub struct UiArgvArgs {
    arg: c_int,
    buffer: *mut c_char,
    buffer_length: c_int,
}

impl UiArgvArgs {
    pub const fn new(arg: c_int, buffer: *mut c_char, buffer_length: c_int) -> Self {
        Self {
            arg,
            buffer,
            buffer_length,
        }
    }

    pub const fn arg(&self) -> c_int {
        self.arg
    }

    pub const fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub const fn buffer_length(&self) -> c_int {
        self.buffer_length
    }
}

impl OutboundSysCall for UiArgv {
    type Import = SpUiImport;
    type Args = UiArgvArgs;
    type Output = ();

    const IMPORT: SpUiImport = SpUiImport::UI_ARGV;
}

impl EncodeSysCall for UiArgv {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.arg() as isize,
            ptr_to_word(args.buffer()),
            args.buffer_length() as isize,
        ])
    }
}

impl DecodeSysCallReturn for UiArgv {
    fn decode_return(_word: isize) -> Self::Output {}
}
