use core::ffi::{c_int, c_void};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::ffi::types::fileHandle_t;

/// Arguments for `CG_FS_READ`.
///
/// Raven wrapper: `return syscall( CG_FS_READ, buffer, len, f );`
/// Raven transport: `FS_Read( VMA(1), args[2], args[3] );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:86-88`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:464-466`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFsReadArgs {
    buffer: *mut c_void,
    len: c_int,
    file: fileHandle_t,
}

impl CgFsReadArgs {
    /// # Safety
    /// `buffer` must be valid for writes of up to `len` bytes.
    pub const unsafe fn new(buffer: *mut c_void, len: c_int, file: fileHandle_t) -> Self {
        Self { buffer, len, file }
    }
}

/// `CG_FS_READ` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:71`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:86-88`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:464-466`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:464-466`
pub struct CgFsRead;

impl OutboundSysCall for CgFsRead {
    type Import = SpCgameImport;
    type Args = CgFsReadArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_FS_READ;
}

impl EncodeSysCall for CgFsRead {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.buffer),
            args.len as isize,
            args.file as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFsRead {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
