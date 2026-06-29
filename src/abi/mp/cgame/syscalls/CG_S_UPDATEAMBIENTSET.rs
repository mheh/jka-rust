use core::ffi::c_char;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_S_UPDATEAMBIENTSET`.
///
/// Raven wrapper: `syscall(CG_S_UPDATEAMBIENTSET, name, origin);`
/// Raven transport: `S_UpdateAmbientSet((const char *)VMA(1), (float *)VMA(2)); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:237-239`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2239`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:846-848`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSUpdateambientsetArgs {
    name: *const c_char,
    origin: *const vec3_t,
}

impl CgSUpdateambientsetArgs {
    pub const fn new(name: *const c_char, origin: *const vec3_t) -> Self {
        Self { name, origin }
    }
}

/// `CG_S_UPDATEAMBIENTSET` MP cgame imports syscall ABI token.
///
/// Raven: rww - AS trap implem
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:110`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:237-239`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:846-848`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:846-848`
pub struct CgSUpdateambientset;

impl OutboundSysCall for CgSUpdateambientset {
    type Import = MpCgameImport;
    type Args = CgSUpdateambientsetArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_UPDATEAMBIENTSET;
}

impl EncodeSysCall for CgSUpdateambientset {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.name), ptr_to_word(args.origin)])
    }
}

impl DecodeSysCallReturn for CgSUpdateambientset {
    fn decode_return(_word: isize) -> Self::Output {}
}
