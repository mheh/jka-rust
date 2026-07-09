use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FX_ADDPOLY`.
///
/// Raven wrapper: `syscall( CG_FX_ADDPOLY, p );`
/// Raven transport opens `addpolyArgStruct_t` from `VMA(1)`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:695-698`
/// Args source: `oracle/codemp/cgame/cg_local.h:2412`
/// Args struct source: `oracle/codemp/game/q_shared.h:2538-2556`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1167-1180`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxAddpolyArgs {
    p: *mut c_void,
}

impl CgFxAddpolyArgs {
    pub const fn new(p: *mut c_void) -> Self {
        Self { p }
    }
}

/// `CG_FX_ADDPOLY` MP cgame imports syscall ABI token.
///
/// Raven: additional funcs for adding custom incode stuff.
/// Enum value source: `oracle/codemp/cgame/cg_public.h:233`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:695-698`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1167-1180`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1167-1180`
pub struct CgFxAddpoly;

impl OutboundSysCall for CgFxAddpoly {
    type Import = MpCgameImport;
    type Args = CgFxAddpolyArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_ADDPOLY;
}

impl EncodeSysCall for CgFxAddpoly {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.p)])
    }
}

impl DecodeSysCallReturn for CgFxAddpoly {
    fn decode_return(_word: isize) -> Self::Output {}
}
