use super::super::MpCgameImport;
use core::ffi::{c_char, c_int};

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_ARGV`.
///
/// Raven cgame calls `syscall( CG_ARGV, n, buffer, bufferLength )`; the MP
/// client switch forwards those as `Cmd_ArgvBuffer(args[1], (char *)VMA(2),
/// args[3])`, writing the selected argv string into the caller-provided buffer.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:75`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:730`
#[derive(Debug)]
pub struct CgArgvArgs {
    n: c_int,
    buffer: *mut c_char,
    buffer_length: c_int,
}

impl CgArgvArgs {
    /// Construct raw `trap_Argv` syscall args.
    ///
    /// # Safety
    /// `buffer` must be valid for writes of up to `buffer_length` bytes.
    pub const unsafe fn new(n: c_int, buffer: *mut c_char, buffer_length: c_int) -> Self {
        Self {
            n,
            buffer,
            buffer_length,
        }
    }

    pub const fn n(&self) -> c_int {
        self.n
    }

    pub const fn buffer(&self) -> *mut c_char {
        self.buffer
    }

    pub const fn buffer_length(&self) -> c_int {
        self.buffer_length
    }
}

/// `CG_ARGV` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:71`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:75`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:732`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:730`
pub struct CgArgv;

impl OutboundSysCall for CgArgv {
    type Import = MpCgameImport;
    type Args = CgArgvArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_ARGV;
}

impl EncodeSysCall for CgArgv {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.n() as isize,
            ptr_to_word(args.buffer()),
            args.buffer_length() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgArgv {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
