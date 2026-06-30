use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::qboolean;

/// Arguments for `CG_CM_LOADMAP`.
///
/// Raven wrapper: `syscall( CG_CM_LOADMAP, mapname, subBSP );`
/// Raven transport: `CL_CM_LoadMap( (const char *) VMA(1), args[2] );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:131-133`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:522-528`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgCmLoadmapArgs {
    mapname: *const c_char,
    sub_bsp: qboolean,
}

impl CgCmLoadmapArgs {
    pub const fn new(mapname: *const c_char, sub_bsp: qboolean) -> Self {
        Self { mapname, sub_bsp }
    }
}

/// `CG_CM_LOADMAP` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:81`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:131-133`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:522-528`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:522-528`
pub struct CgCmLoadmap;

impl OutboundSysCall for CgCmLoadmap {
    type Import = SpCgameImport;
    type Args = CgCmLoadmapArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_CM_LOADMAP;
}

impl EncodeSysCall for CgCmLoadmap {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.mapname), args.sub_bsp as isize])
    }
}

impl DecodeSysCallReturn for CgCmLoadmap {
    fn decode_return(_word: isize) -> Self::Output {}
}
