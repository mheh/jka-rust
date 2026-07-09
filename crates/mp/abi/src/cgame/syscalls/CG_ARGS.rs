use super::super::MpCgameImport;
use core::ffi::{c_char, c_int};

use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_ARGS`.
///
/// Raven cgame calls `syscall( CG_ARGS, buffer, bufferLength )`; the MP client
/// switch forwards those as `Cmd_ArgsBuffer((char *)VMA(1), args[2])`, writing
/// the command args string into the caller-provided buffer.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:79`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:733`
#[derive(Debug)]
pub struct CgArgsArgs {
    buffer: *mut c_char,
    buffer_length: c_int,
}

impl CgArgsArgs {
    /// Construct raw `trap_Args` syscall args.
    ///
    /// # Safety
    /// `buffer` must be valid for writes of up to `buffer_length` bytes.
    pub const unsafe fn new(buffer: *mut c_char, buffer_length: c_int) -> Self {
        Self {
            buffer,
            buffer_length,
        }
    }

    pub const fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub const fn buffer_length(&self) -> c_int {
        self.buffer_length
    }
}

/// `CG_ARGS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:72`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:79`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:735`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:733`
pub struct CgArgs;

impl OutboundSysCall for CgArgs {
    type Import = MpCgameImport;
    type Args = CgArgsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_ARGS;
}

impl EncodeSysCall for CgArgs {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.buffer()), args.buffer_length() as isize])
    }
}

impl DecodeSysCallReturn for CgArgs {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
