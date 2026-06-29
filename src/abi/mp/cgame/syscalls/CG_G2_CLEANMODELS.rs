use core::ffi::c_void;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_G2_CLEANMODELS`.
///
/// Raven wrapper: `syscall(CG_G2_CLEANMODELS, ghoul2Ptr);`
/// Raven transport: `G2API_CleanGhoul2Models((CGhoul2Info_v **)VMA(1)); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:856-858`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2548`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1374-1380`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgG2CleanmodelsArgs {
    ghoul2_ptr: *mut *mut c_void,
}

impl CgG2CleanmodelsArgs {
    pub const fn new(ghoul2_ptr: *mut *mut c_void) -> Self {
        Self { ghoul2_ptr }
    }
}

/// `CG_G2_CLEANMODELS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:267`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:856-858`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1374-1380`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1374-1380`
pub struct CgG2Cleanmodels;

impl OutboundSysCall for CgG2Cleanmodels {
    type Import = MpCgameImport;
    type Args = CgG2CleanmodelsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_G2_CLEANMODELS;
}

impl EncodeSysCall for CgG2Cleanmodels {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.ghoul2_ptr as *const _)])
    }
}

impl DecodeSysCallReturn for CgG2Cleanmodels {
    fn decode_return(_word: isize) -> Self::Output {}
}
