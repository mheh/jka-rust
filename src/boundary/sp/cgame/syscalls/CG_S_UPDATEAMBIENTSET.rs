use core::ffi::c_char;

use super::super::SpCgameImport;
use crate::boundary::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_S_UPDATEAMBIENTSET`.
///
/// Raven wrapper: `syscall( CG_S_UPDATEAMBIENTSET, name, origin );`
/// Raven transport: `S_UpdateAmbientSet((const char *) VMA(1), (float *) VMA(2));`
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:197-198`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:562-569`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSUpdateambientsetArgs {
    name: *const c_char,
    origin: *const vec3_t,
}

impl CgSUpdateambientsetArgs {
    pub const fn new(name: *const c_char, origin: *const vec3_t) -> Self {
        Self { name, origin }
    }

    pub const fn name(&self) -> *const c_char {
        self.name
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }
}

/// `CG_S_UPDATEAMBIENTSET` SP cgame imports syscall boundary token.
///
/// Raven switch comment: stops an `ERR_DROP` internally if called illegally from
/// game side, but can legally arrive during level start before sound has begun.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:162`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:197-198`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:562-569`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:562-569`
pub struct CgSUpdateambientset;

impl OutboundSysCall for CgSUpdateambientset {
    type Import = SpCgameImport;
    type Args = CgSUpdateambientsetArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_UPDATEAMBIENTSET;
}

impl EncodeSysCall for CgSUpdateambientset {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name()), ptr_to_word(args.origin())])
    }
}

impl DecodeSysCallReturn for CgSUpdateambientset {
    fn decode_return(_word: isize) -> Self::Output {}
}
