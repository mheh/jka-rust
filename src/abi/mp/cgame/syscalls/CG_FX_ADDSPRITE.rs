use core::ffi::c_void;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FX_ADDSPRITE`.
///
/// Raven wrapper: `syscall( CG_FX_ADDSPRITE, p );`
/// Raven transport opens `addspriteArgStruct_t` from `VMA(1)`.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:710-713`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2415`
/// Args struct source: `oracle/oracle/codemp/game/q_shared.h:2579-2593`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1210-1229`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxAddspriteArgs {
    p: *mut c_void,
}

impl CgFxAddspriteArgs {
    pub const fn new(p: *mut c_void) -> Self {
        Self { p }
    }
}

/// `CG_FX_ADDSPRITE` MP cgame imports syscall ABI token.
///
/// Raven: additional funcs for adding custom incode stuff.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:236`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:710-713`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1210-1229`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1210-1229`
pub struct CgFxAddsprite;

impl OutboundSysCall for CgFxAddsprite {
    type Import = MpCgameImport;
    type Args = CgFxAddspriteArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_ADDSPRITE;
}

impl EncodeSysCall for CgFxAddsprite {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.p)])
    }
}

impl DecodeSysCallReturn for CgFxAddsprite {
    fn decode_return(_word: isize) -> Self::Output {}
}
