use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_LISTSURFACES`.
///
/// Raven wrapper: `syscall( CG_G2_LISTSURFACES, ghlInfo);`
/// Raven transport: `G2API_ListSurfaces( (CGhoul2Info *) args[1] );`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:771-773`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1296-1298`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2ListsurfacesArgs {
    ghl_info: *mut c_void,
}

impl CgG2ListsurfacesArgs {
    pub const fn new(ghl_info: *mut c_void) -> Self {
        Self { ghl_info }
    }
}

/// `CG_G2_LISTSURFACES` MP cgame imports syscall ABI token.
///
/// Raven: Ghoul2 Insert Start
/// Raven transport: `ghlInfo` is passed as a raw `args[1]` pointer word, not VMA.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:253-256`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:771-773`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1296-1298`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1296-1298`
pub struct CgG2Listsurfaces;

impl OutboundSysCall for CgG2Listsurfaces {
    type Import = MpCgameImport;
    type Args = CgG2ListsurfacesArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_LISTSURFACES;
}

impl EncodeSysCall for CgG2Listsurfaces {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info)])
    }
}

impl DecodeSysCallReturn for CgG2Listsurfaces {
    fn decode_return(_word: isize) -> Self::Output {}
}
