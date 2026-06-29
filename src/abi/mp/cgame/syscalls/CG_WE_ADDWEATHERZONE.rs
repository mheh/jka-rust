use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_WE_ADDWEATHERZONE`.
///
/// Raven: Adding trap to get weather working.
/// Raven wrapper: `syscall( CG_WE_ADDWEATHERZONE, mins, maxs );`
/// Raven transport: `R_AddWeatherZone( (vec_t *)VMA(1), (vec_t *)VMA(2) ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1115-1118`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2429`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1724-1726`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgWeAddweatherzoneArgs {
    mins: *const vec3_t,
    maxs: *const vec3_t,
}

impl CgWeAddweatherzoneArgs {
    pub const fn new(mins: *const vec3_t, maxs: *const vec3_t) -> Self {
        Self { mins, maxs }
    }
}

/// `CG_WE_ADDWEATHERZONE` MP cgame imports syscall ABI token.
///
/// Raven: Adding trap to get weather working
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:336`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:1115-1118`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1724-1726`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1724-1726`
pub struct CgWeAddweatherzone;

impl OutboundSysCall for CgWeAddweatherzone {
    type Import = MpCgameImport;
    type Args = CgWeAddweatherzoneArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_WE_ADDWEATHERZONE;
}

impl EncodeSysCall for CgWeAddweatherzone {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([ptr_to_word(args.mins), ptr_to_word(args.maxs)])
    }
}

impl DecodeSysCallReturn for CgWeAddweatherzone {
    fn decode_return(_word: isize) -> Self::Output {}
}
