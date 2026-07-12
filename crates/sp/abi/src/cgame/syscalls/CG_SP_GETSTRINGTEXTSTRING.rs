use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_SP_GETSTRINGTEXTSTRING`.
///
/// Raven wrapper: `return syscall( CG_SP_GETSTRINGTEXTSTRING, text, buffer, bufferLength );`
/// Raven transport: `text = SE_GetString((const char *) VMA(1));`, then copies into `VMA(2)` with size `args[3]` and returns `strlen(text)`.
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:640`
/// Args source: `oracle/code/cgame/cg_local.h:1206`
/// Output source: `oracle/code/client/cl_cgame.cpp:995-1010`
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CgSpGetstringtextstringArgs {
    text: *const c_char,
    buffer: *mut c_char,
    buffer_length: c_int,
}

impl CgSpGetstringtextstringArgs {
    pub const fn new(text: *const c_char, buffer: *mut c_char, buffer_length: c_int) -> Self {
        Self {
            text,
            buffer,
            buffer_length,
        }
    }

    pub const fn text(&self) -> *const c_char {
        self.text
    }

    pub const fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub const fn buffer_length(&self) -> c_int {
        self.buffer_length
    }
}

/// `CG_SP_GETSTRINGTEXTSTRING` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:206`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:640`
/// Output source: `oracle/code/client/cl_cgame.cpp:995-1010`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:995-1010`
pub struct CgSpGetstringtextstring;

impl OutboundSysCall for CgSpGetstringtextstring {
    type Import = SpCgameImport;
    type Args = CgSpGetstringtextstringArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_SP_GETSTRINGTEXTSTRING;
}

impl EncodeSysCall for CgSpGetstringtextstring {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.text()),
            ptr_to_word(args.buffer()),
            args.buffer_length() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSpGetstringtextstring {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
