use core::ffi::c_int;

use super::super::SpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use sp_qshared::shared::vec3_t;

/// Arguments for `CG_S_ADDLOOPINGSOUND`.
///
/// Raven wrapper: `syscall( CG_S_ADDLOOPINGSOUND, entityNum, origin, velocity, sfx, chan );`
/// Raven transport: `S_AddLoopingSound( args[1], (const float *) VMA(2), (const float *) VMA(3), args[4], (soundChannel_t)args[5] );`
///
/// Raven comment: stops an ERR_DROP internally if called illegally from game side, but note
/// that it also gets here legally during level start where normally the internal
/// `s_soundStarted` check would return.
///
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:217-218`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:591-598`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgSAddloopingsoundArgs {
    entity_num: c_int,
    origin: *const vec3_t,
    velocity: *const vec3_t,
    sfx: c_int,
    chan: c_int,
}

impl CgSAddloopingsoundArgs {
    pub const fn new(
        entity_num: c_int,
        origin: *const vec3_t,
        velocity: *const vec3_t,
        sfx: c_int,
        chan: c_int,
    ) -> Self {
        Self {
            entity_num,
            origin,
            velocity,
            sfx,
            chan,
        }
    }

    pub const fn entity_num(&self) -> c_int {
        self.entity_num
    }

    pub const fn origin(&self) -> *const vec3_t {
        self.origin
    }

    pub const fn velocity(&self) -> *const vec3_t {
        self.velocity
    }

    pub const fn sfx(&self) -> c_int {
        self.sfx
    }

    pub const fn chan(&self) -> c_int {
        self.chan
    }
}

/// `CG_S_ADDLOOPINGSOUND` SP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/code/cgame/cg_public.h:94`
/// Args source: `oracle/oracle/code/cgame/cg_syscalls.cpp:217-218`
/// Output source: `oracle/oracle/code/client/cl_cgame.cpp:591-598`
/// Transport/switch source: `oracle/oracle/code/client/cl_cgame.cpp:591-598`
pub struct CgSAddloopingsound;

impl OutboundSysCall for CgSAddloopingsound {
    type Import = SpCgameImport;
    type Args = CgSAddloopingsoundArgs;
    type Output = ();

    const IMPORT: SpCgameImport = SpCgameImport::CG_S_ADDLOOPINGSOUND;
}

impl EncodeSysCall for CgSAddloopingsound {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.entity_num() as isize,
            ptr_to_word(args.origin()),
            ptr_to_word(args.velocity()),
            args.sfx() as isize,
            args.chan() as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgSAddloopingsound {
    fn decode_return(_word: isize) -> Self::Output {}
}
