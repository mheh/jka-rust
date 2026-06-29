use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FX_INIT_SYSTEM`.
///
/// Raven wrapper: `return syscall( CG_FX_INIT_SYSTEM, refdef );`
/// Raven transport: `return FX_InitSystem( (refdef_t*)VMA(1) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:669-671`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2396`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1149-1150`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxInitSystemArgs {
    refdef: *mut c_void,
}

impl CgFxInitSystemArgs {
    pub const fn new(refdef: *mut c_void) -> Self {
        Self { refdef }
    }
}

/// `CG_FX_INIT_SYSTEM` MP cgame imports syscall boundary token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:227`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:669-671`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1149-1150`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1149-1150`
pub struct CgFxInitSystem;

impl OutboundSysCall for CgFxInitSystem {
    type Import = MpCgameImport;
    type Args = CgFxInitSystemArgs;
    type Output = c_int;

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_INIT_SYSTEM;
}

impl EncodeSysCall for CgFxInitSystem {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.refdef)])
    }
}

impl DecodeSysCallReturn for CgFxInitSystem {
    fn decode_return(word: isize) -> Self::Output {
        word as c_int
    }
}
