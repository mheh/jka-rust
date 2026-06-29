use core::ffi::c_char;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_RE_INIT_RENDERER_TERRAIN`.
///
/// Raven: rwwRMG - added [NEWTRAP].
/// Raven wrapper: `syscall(CG_RE_INIT_RENDERER_TERRAIN, info);`
/// Raven transport: `RE_InitRendererTerrain((const char *)VMA(1)); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1102-1104`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2437`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1712-1714`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgReInitRendererTerrainArgs {
    info: *const c_char,
}

impl CgReInitRendererTerrainArgs {
    pub const fn new(info: *const c_char) -> Self {
        Self { info }
    }
}

/// `CG_RE_INIT_RENDERER_TERRAIN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:332`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1102-1104`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1712-1714`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1712-1714`
pub struct CgReInitRendererTerrain;

impl OutboundSysCall for CgReInitRendererTerrain {
    type Import = MpCgameImport;
    type Args = CgReInitRendererTerrainArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_RE_INIT_RENDERER_TERRAIN;
}

impl EncodeSysCall for CgReInitRendererTerrain {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.info)])
    }
}

impl DecodeSysCallReturn for CgReInitRendererTerrain {
    fn decode_return(_word: isize) -> Self::Output {}
}
