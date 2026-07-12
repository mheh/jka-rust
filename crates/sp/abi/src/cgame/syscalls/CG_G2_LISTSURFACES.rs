use core::ffi::c_void;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_LISTSURFACES`.
///
/// Raven wrapper: `syscall( CG_G2_LISTSURFACES, ghlInfo);`
/// Raven transport: `G2API_ListSurfaces( (CGhoul2Info *) args[1] );`
///
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:783-784`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:783-785`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2ListsurfacesArgs {
    ghl_info: *mut c_void,
}

impl CgG2ListsurfacesArgs {
    pub const fn new(ghl_info: *mut c_void) -> Self {
        Self { ghl_info }
    }
}

/// `CG_G2_LISTSURFACES` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/code/cgame/cg_public.h:173`
/// Args source: `oracle/code/cgame/cg_syscalls.cpp:783-784`
/// Output source: `oracle/code/client/cl_cgame.cpp:783-785`
/// Transport/switch source: `oracle/code/client/cl_cgame.cpp:783-785`
pub struct CgG2Listsurfaces;

impl OutboundSysCall for CgG2Listsurfaces {
    type Import = SpCgameImport;
    type Args = CgG2ListsurfacesArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_G2_LISTSURFACES;
}

impl EncodeSysCall for CgG2Listsurfaces {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghl_info)])
    }
}

impl DecodeSysCallReturn for CgG2Listsurfaces {
    fn decode_return(_word: isize) -> Self::Output {}
}
