use core::ffi::c_void;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};

/// Arguments for `CG_FX_ADDBEZIER`.
///
/// Raven wrapper: `syscall( CG_FX_ADDBEZIER, p );`
/// Raven transport opens `addbezierArgStruct_t` from `VMA(1)`.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:700-703`
/// Args source: `oracle/codemp/cgame/cg_local.h:2413`
/// Args struct source: `oracle/codemp/game/q_shared.h:2558-2577`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1182-1195`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxAddbezierArgs {
    p: *mut c_void,
}

impl CgFxAddbezierArgs {
    pub const fn new(p: *mut c_void) -> Self {
        Self { p }
    }
}

/// `CG_FX_ADDBEZIER` MP cgame imports syscall ABI token.
///
/// Raven: additional funcs for adding custom incode stuff.
/// Enum value source: `oracle/codemp/cgame/cg_public.h:234`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:700-703`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1182-1195`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1182-1195`
pub struct CgFxAddbezier;

impl OutboundSysCall for CgFxAddbezier {
    type Import = MpCgameImport;
    type Args = CgFxAddbezierArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_ADDBEZIER;
}

impl EncodeSysCall for CgFxAddbezier {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.p)])
    }
}

impl DecodeSysCallReturn for CgFxAddbezier {
    fn decode_return(_word: isize) -> Self::Output {}
}
