use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FX_SET_REFDEF`.
///
/// Raven wrapper: `syscall( CG_FX_SET_REFDEF, refdef );`
/// Raven transport: `FX_SetRefDefFromCGame( (refdef_t*)VMA(1) ); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:674-676`
/// Args source: `oracle/codemp/cgame/cg_local.h:2397`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1152-1154`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxSetRefdefArgs {
    refdef: *mut c_void,
}

impl CgFxSetRefdefArgs {
    pub const fn new(refdef: *mut c_void) -> Self {
        Self { refdef }
    }
}

/// `CG_FX_SET_REFDEF` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:228`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:674-676`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1152-1154`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1152-1154`
pub struct CgFxSetRefdef;

impl OutboundSysCall for CgFxSetRefdef {
    type Import = MpCgameImport;
    type Args = CgFxSetRefdefArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_SET_REFDEF;
}

impl EncodeSysCall for CgFxSetRefdef {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.refdef)])
    }
}

impl DecodeSysCallReturn for CgFxSetRefdef {
    fn decode_return(_word: isize) -> Self::Output {}
}
