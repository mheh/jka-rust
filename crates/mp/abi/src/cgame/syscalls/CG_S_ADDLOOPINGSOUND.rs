use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_S_ADDLOOPINGSOUND`.
///
/// Raven wrapper: `syscall( CG_S_ADDLOOPINGSOUND, entityNum, origin, velocity, sfx );`
/// Raven transport: `S_AddLoopingSound( args[1], (const float *)VMA(2), (const float *)VMA(3), args[4] );`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:204-205`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2227`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:821-823`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSAddloopingsoundArgs {
    entity_num: c_int,
    origin: *const vec3_t,
    velocity: *const vec3_t,
    sfx: c_int,
}

impl CgSAddloopingsoundArgs {
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

/// `CG_S_ADDLOOPINGSOUND` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:100`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:204-205`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:821-823`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:821-823`
pub struct CgSAddloopingsound;

impl OutboundSysCall for CgSAddloopingsound {
    type Import = MpCgameImport;
    type Args = CgSAddloopingsoundArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_S_ADDLOOPINGSOUND;
}

impl EncodeSysCall for CgSAddloopingsound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.entity_num as isize,
            ptr_to_word(args.origin),
            ptr_to_word(args.velocity),
            args.sfx as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSAddloopingsound {
    fn decode_return(_word: isize) -> Self::Output {}
}
