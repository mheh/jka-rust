use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_RMG_INIT`.
///
/// Raven: rwwRMG - added [NEWTRAP].
/// Raven wrapper: `syscall(CG_RMG_INIT, terrainID, terrainInfo);`
/// Raven transport: `RM_CreateRandomModels(args[1], (const char *)VMA(2)); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1097-1099`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2436`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1689-1710`
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

/// `CG_RMG_INIT` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:331`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1097-1099`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1689-1710`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1689-1710`
pub struct CgRmgInit;

impl OutboundSysCall for CgRmgInit {
    type Import = MpCgameImport;
    type Args = CgRmgInitArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_RMG_INIT;
}

impl EncodeSysCall for CgRmgInit {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.terrain_id as isize, ptr_to_word(args.terrain_info)])
    }
}

impl DecodeSysCallReturn for CgRmgInit {
    fn decode_return(_word: isize) -> Self::Output {}
}
