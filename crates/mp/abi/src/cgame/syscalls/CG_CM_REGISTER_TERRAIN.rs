use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_CM_REGISTER_TERRAIN`.
///
/// Raven: rwwRMG - added [NEWTRAP].
/// Raven wrapper: `return syscall(CG_CM_REGISTER_TERRAIN, config);`
/// Raven transport: `return CM_RegisterTerrain((const char *)VMA(1), false)->GetTerrainId();`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1092-1094`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2435`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1686-1687`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmRegisterTerrainArgs {
    config: *const c_char,
}

impl CgCmRegisterTerrainArgs {
    pub const fn new(config: *const c_char) -> Self {
        Self { config }
    }
}

/// `CG_CM_REGISTER_TERRAIN` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:330`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1092-1094`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1686-1687`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1686-1687`
pub struct CgCmRegisterTerrain;

impl OutboundSysCall for CgCmRegisterTerrain {
    type Import = MpCgameImport;
    type Args = CgCmRegisterTerrainArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_CM_REGISTER_TERRAIN;
}

impl EncodeSysCall for CgCmRegisterTerrain {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.config)])
    }
}

impl DecodeSysCallReturn for CgCmRegisterTerrain {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
