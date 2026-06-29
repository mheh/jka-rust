use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_ARGV`.
///
/// Raven wrapper: `syscall( CG_ARGV, n, buffer, bufferLength );`
/// Raven transport: `Cmd_ArgvBuffer( args[1], (char *) VMA(2), args[3] );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:74-76`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:456-458`
#[derive(Debug)]
pub struct CgArgvArgs {
    n: c_int,
    buffer: *mut c_char,
    buffer_length: c_int,
}

impl CgArgvArgs {
    /// # Safety
    /// `buffer` must be valid for writes of up to `buffer_length` bytes.
    pub const unsafe fn new(n: c_int, buffer: *mut c_char, buffer_length: c_int) -> Self {
        Self {
            n,
            buffer,
            buffer_length,
        }
    }
}

/// `CG_ARGV` SP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:68`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:74-76`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:456-458`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:456-458`
pub struct CgArgv;

impl OutboundSysCall for CgArgv {
    type Import = SpCgameImport;
    type Args = CgArgvArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_ARGV;
}

impl EncodeSysCall for CgArgv {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.n as isize,
            ptr_to_word(args.buffer),
            args.buffer_length as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgArgv {
    fn decode_return(_word: isize) -> Self::Output {}
}
