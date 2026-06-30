use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::fileHandle_t;

/// Arguments for the `CG_FS_WRITE` outbound cgame-to-engine syscall.
///
/// Raven wrapper: `trap_FS_Write(const void *buffer, int len, fileHandle_t f)`
/// forwards the raw buffer pointer, byte count, and file handle.
///
/// Sources:
/// - args: `oracle/oracle/codemp/cgame/cg_syscalls.c:91-92`
/// - transport: `oracle/oracle/codemp/client/cl_cgame.cpp:742-743`
#[derive(Debug)]
pub struct CgFsWriteArgs {
    buffer: *const u8,
    len: c_int,
    f: fileHandle_t,
}

impl CgFsWriteArgs {
    pub fn new(buffer: *const u8, len: c_int, f: fileHandle_t) -> Self {
        Self { buffer, len, f }
    }

    pub fn buffer(&self) -> *const u8 {
        self.buffer
    }

    pub fn len(&self) -> c_int {
        self.len
    }

    pub fn f(&self) -> fileHandle_t {
        self.f
    }
}

/// `CG_FS_WRITE` MP cgame imports syscall ABI token.
///
/// Raven: `( const void *buffer, int len, fileHandle_t f );`
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:75`
pub struct CgFsWrite;

impl OutboundSysCall for CgFsWrite {
    type Import = MpCgameImport;
    type Args = CgFsWriteArgs;
    /// Output: wrapper is `void`; switch arm calls `FS_Write(...)` and returns
    /// `0` to the VM.
    ///
    /// Sources:
    /// - wrapper output: `oracle/oracle/codemp/cgame/cg_syscalls.c:91-92`
    /// - switch output: `oracle/oracle/codemp/client/cl_cgame.cpp:742-744`
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FS_WRITE;
}

impl EncodeSysCall for CgFsWrite {
    fn encode_syscall(a: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(a.buffer), a.len as isize, a.f as isize])
    }
}

impl DecodeSysCallReturn for CgFsWrite {
    fn decode_return(_word: isize) -> Self::Output {
        ()
    }
}
