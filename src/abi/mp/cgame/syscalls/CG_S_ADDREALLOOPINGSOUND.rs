use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::vec3_t;

/// Arguments for `CG_S_ADDREALLOOPINGSOUND`.
///
/// Raven wrapper: `syscall( CG_S_ADDREALLOOPINGSOUND, entityNum, origin, velocity, sfx );`
/// Raven transport calls `S_AddLoopingSound`, with `S_AddRealLoopingSound` commented out.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:212-213`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2228`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:824-828`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSAddrealloopingsoundArgs {
    entity_num: c_int,
    origin: *const vec3_t,
    velocity: *const vec3_t,
    sfx: c_int,
}

impl CgSAddrealloopingsoundArgs {
    pub const fn new(
        entity_num: c_int,
        origin: *const vec3_t,
        velocity: *const vec3_t,
        sfx: c_int,
    ) -> Self {
        Self {
            entity_num,
            origin,
            velocity,
            sfx,
        }
    }
}

/// `CG_S_ADDREALLOOPINGSOUND` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:102`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:212-213`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:824-828`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:824-828`
pub struct CgSAddrealloopingsound;

impl OutboundSysCall for CgSAddrealloopingsound {
    type Import = MpCgameImport;
    type Args = CgSAddrealloopingsoundArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_ADDREALLOOPINGSOUND;
}

impl EncodeSysCall for CgSAddrealloopingsound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.entity_num as isize,
            ptr_to_word(args.origin),
            ptr_to_word(args.velocity),
            args.sfx as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSAddrealloopingsound {
    fn decode_return(_word: isize) -> Self::Output {}
}
