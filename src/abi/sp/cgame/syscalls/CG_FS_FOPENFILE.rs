use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::fileHandle_t;
use crate::shared::fsMode_t;

/// Arguments for `CG_FS_FOPENFILE`.
///
/// Raven wrapper: `return syscall( CG_FS_FOPENFILE, qpath, f, mode );`
/// Raven transport: `return FS_FOpenFileByMode( (const char *) VMA(1), (int *) VMA(2), (fsMode_t) args[3] );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:82-84`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:462-463`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFsFopenfileArgs {
    qpath: *const c_char,
    file: *mut fileHandle_t,
    mode: fsMode_t,
}

impl CgFsFopenfileArgs {
    /// # Safety
    /// `file` must point to a valid writable `fileHandle_t`.
    pub const unsafe fn new(qpath: *const c_char, file: *mut fileHandle_t, mode: fsMode_t) -> Self {
        Self { qpath, file, mode }
    }
}

/// `CG_FS_FOPENFILE` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:70`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:82-84`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:462-463`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:462-463`
pub struct CgFsFopenfile;

impl OutboundSysCall for CgFsFopenfile {
    type Import = SpCgameImport;
    type Args = CgFsFopenfileArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_FS_FOPENFILE;
}

impl EncodeSysCall for CgFsFopenfile {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.qpath),
            ptr_to_word(args.file),
            args.mode as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFsFopenfile {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
