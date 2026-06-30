use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::types::qboolean;

/// Arguments for `CG_ROFF_PURGE_ENT`.
///
/// Raven wrapper: `return syscall( CG_ROFF_PURGE_ENT, entID );`
/// Raven transport: `return theROFFSystem.PurgeEnt( args[1], qtrue );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:750-752`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2434`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1281-1282`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRoffPurgeEntArgs {
    ent_id: c_int,
}

impl CgRoffPurgeEntArgs {
    pub const fn new(ent_id: c_int) -> Self {
        Self { ent_id }
    }
}

/// `CG_ROFF_PURGE_ENT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:246`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:750-752`
/// Output source: `oracle/oracle/codemp/cgame/cg_syscalls.c:750-752`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1281-1282`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1281-1282`
pub struct CgRoffPurgeEnt;

impl OutboundSysCall for CgRoffPurgeEnt {
    type Import = MpCgameImport;
    type Args = CgRoffPurgeEntArgs;
    type Output = qboolean;

    const IMPORT: MpCgameImport = MpCgameImport::CG_ROFF_PURGE_ENT;
}

impl EncodeSysCall for CgRoffPurgeEnt {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.ent_id as isize])
    }
}

impl DecodeSysCallReturn for CgRoffPurgeEnt {
    fn decode_return(word: isize) -> Self::Output {
        word as qboolean
    }
}
