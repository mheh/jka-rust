use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_RMG_INIT`.
///
/// Raven wrapper: `syscall( CG_RMG_INIT, terrainID, terrainInfo);`
/// Raven transport: `RM_CreateRandomModels(args[1], (const char *)VMA(2));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:115-118`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:494-513`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgRmgInitArgs {
    terrain_id: c_int,
    terrain_info: *const c_char,
}

impl CgRmgInitArgs {
    pub const fn new(terrain_id: c_int, terrain_info: *const c_char) -> Self {
        Self {
            terrain_id,
            terrain_info,
        }
    }
}

/// `CG_RMG_INIT` SP cgame imports syscall boundary token.
///
/// Raven: RMG BEGIN
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:78`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:115-118`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:494-513`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:494-513`
pub struct CgRmgInit;

impl OutboundSysCall for CgRmgInit {
    type Import = SpCgameImport;
    type Args = CgRmgInitArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_RMG_INIT;
}

impl EncodeSysCall for CgRmgInit {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.terrain_id as isize, ptr_to_word(args.terrain_info)])
    }
}

impl DecodeSysCallReturn for CgRmgInit {
    fn decode_return(_word: isize) -> Self::Output {}
}
