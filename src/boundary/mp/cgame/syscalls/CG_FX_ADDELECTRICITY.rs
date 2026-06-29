use core::ffi::c_void;

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FX_ADDELECTRICITY`.
///
/// Raven wrapper: `syscall( CG_FX_ADDELECTRICITY, p );`
/// Raven transport opens `addElectricityArgStruct_t` from `VMA(1)`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:715-718`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2416`
/// Args struct source: `oracle/oracle/codemp/game/q_shared.h:2622-2639`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1230-1242`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxAddelectricityArgs {
    p: *mut c_void,
}

impl CgFxAddelectricityArgs {
    pub const fn new(p: *mut c_void) -> Self {
        Self { p }
    }
}

/// `CG_FX_ADDELECTRICITY` MP cgame imports syscall boundary token.
///
/// Raven: additional funcs for adding custom incode stuff.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:237`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:715-718`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1230-1242`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1230-1242`
pub struct CgFxAddelectricity;

impl OutboundSysCall for CgFxAddelectricity {
    type Import = MpCgameImport;
    type Args = CgFxAddelectricityArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_ADDELECTRICITY;
}

impl EncodeSysCall for CgFxAddelectricity {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.p)])
    }
}

impl DecodeSysCallReturn for CgFxAddelectricity {
    fn decode_return(_word: isize) -> Self::Output {}
}
