use core::ffi::{c_int, c_void};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::fileHandle_t;

/// Arguments for `CG_FS_WRITE`.
///
/// Raven wrapper: `return syscall( CG_FS_WRITE, buffer, len, f );`
/// Raven transport: `FS_Write( VMA(1), args[2], args[3] );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:90-92`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:467-469`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFsWriteArgs {
    buffer: *const c_void,
    len: c_int,
    file: fileHandle_t,
}

impl CgFsWriteArgs {
    pub const fn new(buffer: *const c_void, len: c_int, file: fileHandle_t) -> Self {
        Self { buffer, len, file }
    }
}

/// `CG_FS_WRITE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:72`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:90-92`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:467-469`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:467-469`
pub struct CgFsWrite;

impl OutboundSysCall for CgFsWrite {
    type Import = SpCgameImport;
    type Args = CgFsWriteArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_FS_WRITE;
}

impl EncodeSysCall for CgFsWrite {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.buffer),
            args.len as isize,
            args.file as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFsWrite {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
