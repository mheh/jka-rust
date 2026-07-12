use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_ARGS`.
///
/// Raven wrapper: `syscall( CG_ARGS, buffer, bufferLength );`
/// Raven transport: `Cmd_ArgsBuffer( (char *) VMA(1), args[2] );`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:78-80`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:459-461`
#[derive(Debug)]
pub struct CgArgsArgs {
    buffer: *mut c_char,
    buffer_length: c_int,
}

impl CgArgsArgs {
    /// # Safety
    /// `buffer` must be valid for writes of up to `buffer_length` bytes.
    pub const unsafe fn new(buffer: *mut c_char, buffer_length: c_int) -> Self {
        Self {
            buffer,
            buffer_length,
        }
    }
}

/// `CG_ARGS` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:69`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:78-80`
/// Output source: `oracle/code/client/cl_cgame.cpp:459-461`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:459-461`
pub struct CgArgs;

impl OutboundSysCall for CgArgs {
    type Import = SpCgameImport;
    type Args = CgArgsArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_ARGS;
}

impl EncodeSysCall for CgArgs {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.buffer), args.buffer_length as isize])
    }
}

impl DecodeSysCallReturn for CgArgs {
    fn decode_return(_word: isize) -> Self::Output {}
}
