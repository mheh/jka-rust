use core::ffi::c_int;

use super::super::SpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_S_UPDATEENTITYPOSITION`.
///
/// Raven wrapper: `syscall( CG_S_UPDATEENTITYPOSITION, entityNum, origin );`
/// Raven transport: `S_UpdateEntityPosition( args[1], (const float *)VMA(2) );`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:222-223`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:599-601`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSUpdateentitypositionArgs {
    entity_num: c_int,
    origin: *const vec3_t,
}

impl CgSUpdateentitypositionArgs {
    pub const fn new(entity_num: c_int, origin: *const vec3_t) -> Self {
        Self { entity_num, origin }
    }

    pub const fn entity_num(&self) -> c_int {
        self.entity_num
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }
}

/// `CG_S_UPDATEENTITYPOSITION` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:96`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:222-223`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:599-601`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:599-601`
pub struct CgSUpdateentityposition;

impl OutboundSysCall for CgSUpdateentityposition {
    type Import = SpCgameImport;
    type Args = CgSUpdateentitypositionArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_UPDATEENTITYPOSITION;
}

impl EncodeSysCall for CgSUpdateentityposition {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.entity_num() as isize, ptr_to_word(args.origin())])
    }
}

impl DecodeSysCallReturn for CgSUpdateentityposition {
    fn decode_return(_word: isize) -> Self::Output {}
}
