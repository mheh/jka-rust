use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_RE_INIT_RENDERER_TERRAIN`.
///
/// Raven wrapper: `syscall(CG_RE_INIT_RENDERER_TERRAIN, terrainInfo);`
/// Raven transport: `RE_InitRendererTerrain((const char *)VMA(1));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:125-128`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:517-519`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgReInitRendererTerrainArgs {
    terrain_info: *const c_char,
}

impl CgReInitRendererTerrainArgs {
    pub const fn new(terrain_info: *const c_char) -> Self {
        Self { terrain_info }
    }
}

/// `CG_RE_INIT_RENDERER_TERRAIN` SP cgame imports syscall ABI token.
///
/// Raven: RMG END
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:80`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:125-128`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:517-519`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:517-519`
pub struct CgReInitRendererTerrain;

impl OutboundSysCall for CgReInitRendererTerrain {
    type Import = SpCgameImport;
    type Args = CgReInitRendererTerrainArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_RE_INIT_RENDERER_TERRAIN;
}

impl EncodeSysCall for CgReInitRendererTerrain {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.terrain_info)])
    }
}

impl DecodeSysCallReturn for CgReInitRendererTerrain {
    fn decode_return(_word: isize) -> Self::Output {}
}
