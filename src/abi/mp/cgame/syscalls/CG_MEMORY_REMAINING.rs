use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};

/// Arguments for `CG_MEMORY_REMAINING`.
///
/// Raven wrapper: `return syscall( CG_MEMORY_REMAINING );`
/// Raven transport: `return Hunk_MemoryRemaining();`
///
/// Raven comment: `aids for VM testing`.
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:521-522`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2356-2360`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:987-988`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgMemoryRemainingArgs;

impl CgMemoryRemainingArgs {
    pub const fn new() -> Self {
        Self
    }
}

/// `CG_MEMORY_REMAINING` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:193`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:521-522`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:987-988`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:987-988`
pub struct CgMemoryRemaining;

impl OutboundSysCall for CgMemoryRemaining {
    type Import = MpCgameImport;
    type Args = CgMemoryRemainingArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_MEMORY_REMAINING;
}

impl EncodeSysCall for CgMemoryRemaining {
    fn encode_syscall(_args: &Self::Args) -> SysCallTransport {
        SysCallTransport::empty()
    }
}

impl DecodeSysCallReturn for CgMemoryRemaining {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
