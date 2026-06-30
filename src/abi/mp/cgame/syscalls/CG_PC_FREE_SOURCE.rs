use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_PC_FREE_SOURCE`.
///
/// Raven wrapper: `return syscall( CG_PC_FREE_SOURCE, handle );`
/// Raven transport: `return botlib_export->PC_FreeSourceHandle( args[1] );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:549-550`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1003-1004`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgPcFreeSourceArgs {
    handle: c_int,
}

impl CgPcFreeSourceArgs {
    pub const fn new(handle: c_int) -> Self {
        Self { handle }
    }
}

/// `CG_PC_FREE_SOURCE` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:201`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:549-550`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1003-1004`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1003-1004`
pub struct CgPcFreeSource;

impl OutboundSysCall for CgPcFreeSource {
    type Import = MpCgameImport;
    type Args = CgPcFreeSourceArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_PC_FREE_SOURCE;
}

impl EncodeSysCall for CgPcFreeSource {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.handle as isize])
    }
}

impl DecodeSysCallReturn for CgPcFreeSource {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
