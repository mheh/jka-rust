use core::ffi::{c_char, c_int};

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CM_REGISTER_TERRAIN`.
///
/// Raven wrapper: `return syscall( CG_CM_REGISTER_TERRAIN, terrainInfo);`
/// Raven transport: `return CM_RegisterTerrain((const char *)VMA(1), false)->GetTerrainId();`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:120-123`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:514-515`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmRegisterTerrainArgs {
    terrain_info: *const c_char,
}

impl CgCmRegisterTerrainArgs {
    pub const fn new(terrain_info: *const c_char) -> Self {
        Self { terrain_info }
    }
}

/// `CG_CM_REGISTER_TERRAIN` SP cgame imports syscall boundary token.
///
/// Raven: RMG BEGIN
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:79`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:120-123`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:514-515`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:514-515`
pub struct CgCmRegisterTerrain;

impl OutboundSysCall for CgCmRegisterTerrain {
    type Import = SpCgameImport;
    type Args = CgCmRegisterTerrainArgs;
    type Output = c_int;

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_REGISTER_TERRAIN;
}

impl EncodeSysCall for CgCmRegisterTerrain {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.terrain_info)])
    }
}

impl DecodeSysCallReturn for CgCmRegisterTerrain {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
