use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_ROFF_CACHE`.
///
/// Raven wrapper: `return syscall( CG_ROFF_CACHE, file );`
/// Raven transport: `return theROFFSystem.Cache( (char *)VMA(1), qtrue );`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:740-742`
/// Args source: `oracle/codemp/cgame/cg_local.h:2432`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1275-1276`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRoffCacheArgs {
    file: *mut c_char,
}

impl CgRoffCacheArgs {
    pub const fn new(file: *mut c_char) -> Self {
        Self { file }
    }
}

/// `CG_ROFF_CACHE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:244`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:740-742`
/// Output source: `oracle/codemp/cgame/cg_syscalls.c:740-742`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1275-1276`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1275-1276`
pub struct CgRoffCache;

impl OutboundSysCall for CgRoffCache {
    type Import = MpCgameImport;
    type Args = CgRoffCacheArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_ROFF_CACHE;
}

impl EncodeSysCall for CgRoffCache {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.file)])
    }
}

impl DecodeSysCallReturn for CgRoffCache {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
