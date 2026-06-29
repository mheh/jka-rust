use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_S_UPDATEENTITYPOSITION`.
///
/// Raven wrapper: `syscall( CG_S_UPDATEENTITYPOSITION, entityNum, origin );`
/// Raven transport: `S_UpdateEntityPosition( args[1], (const float *)VMA(2) );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:208-209`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2229`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:831-833`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSUpdateentitypositionArgs {
    entity_num: c_int,
    origin: *const vec3_t,
}

impl CgSUpdateentitypositionArgs {
    pub const fn new(entity_num: c_int, origin: *const vec3_t) -> Self {
        Self { entity_num, origin }
    }
}

/// `CG_S_UPDATEENTITYPOSITION` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:101`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:208-209`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:831-833`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:831-833`
pub struct CgSUpdateentityposition;

impl OutboundSysCall for CgSUpdateentityposition {
    type Import = MpCgameImport;
    type Args = CgSUpdateentitypositionArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_UPDATEENTITYPOSITION;
}

impl EncodeSysCall for CgSUpdateentityposition {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.entity_num as isize, ptr_to_word(args.origin)])
    }
}

impl DecodeSysCallReturn for CgSUpdateentityposition {
    fn decode_return(_word: isize) -> Self::Output {}
}
