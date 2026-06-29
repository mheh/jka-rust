use core::ffi::c_void;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FX_ADDPRIMITIVE`.
///
/// Raven wrapper: `syscall( CG_FX_ADDPRIMITIVE, p );`
/// Raven transport opens `effectTrailArgStruct_t` from `VMA(1)`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:705-708`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2414`
/// Args struct source: `oracle/oracle/codemp/game/q_shared.h:2615-2620`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1197-1208`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxAddprimitiveArgs {
    p: *mut c_void,
}

impl CgFxAddprimitiveArgs {
    pub const fn new(p: *mut c_void) -> Self {
        Self { p }
    }
}

/// `CG_FX_ADDPRIMITIVE` MP cgame imports syscall boundary token.
///
/// Raven: additional funcs for adding custom incode stuff.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:235`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:705-708`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1197-1208`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1197-1208`
pub struct CgFxAddprimitive;

impl OutboundSysCall for CgFxAddprimitive {
    type Import = MpCgameImport;
    type Args = CgFxAddprimitiveArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_ADDPRIMITIVE;
}

impl EncodeSysCall for CgFxAddprimitive {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.p)])
    }
}

impl DecodeSysCallReturn for CgFxAddprimitive {
    fn decode_return(_word: isize) -> Self::Output {}
}
