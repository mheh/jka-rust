use super::super::MpCgameImport;
use core::ffi::c_int;

use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::fileHandle_t;

/// Arguments for `CG_FS_READ`.
///
/// Raven cgame calls `syscall( CG_FS_READ, buffer, len, f )`; the MP client
/// switch decodes `buffer` through `VMA(1)`, reads `len` and `f` from
/// `args[2]`/`args[3]`, and returns `0` after filling the caller-owned buffer.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:87-88`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:739-741`
#[derive(Debug)]
pub struct CgFsReadArgs {
    buffer: *mut u8,
    len: c_int,
    f: fileHandle_t,
}

impl CgFsReadArgs {
    /// Construct raw `trap_FS_Read` syscall args.
    ///
    /// # Safety
    /// `buffer` must be valid for writes of up to `len` bytes for the duration
    /// of the syscall.
    pub const unsafe fn new(buffer: *mut u8, len: c_int, f: fileHandle_t) -> Self {
        Self { buffer, len, f }
    }

    pub const fn buffer(&self) -> *mut u8 {
        self.buffer
    }

    pub const fn len(&self) -> c_int {
        self.len
    }

    pub const fn f(&self) -> fileHandle_t {
        self.f
    }
}

/// `CG_FS_READ` MP cgame imports syscall ABI token.
///
/// Raven: `( void *buffer, int len, fileHandle_t f )`.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:74`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:87-88`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:741`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:739-741`
pub struct CgFsRead;

impl OutboundSysCall for CgFsRead {
    type Import = MpCgameImport;
    type Args = CgFsReadArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FS_READ;
}

impl EncodeSysCall for CgFsRead {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.buffer() as *const u8),
            args.len() as isize,
            args.f() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFsRead {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
