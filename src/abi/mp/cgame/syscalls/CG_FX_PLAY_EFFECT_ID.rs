use core::ffi::c_int;

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::codemp::game::q_shared_h::vec3_t;

/// Arguments for `CG_FX_PLAY_EFFECT_ID`.
///
/// Raven wrapper: `syscall( CG_FX_PLAY_EFFECT_ID, id, org, fwd, vol, rad );`
/// Raven transport: `FX_PlayEffectID(args[1], (float *)VMA(2), (float *)VMA(3), args[4], args[5] ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:637-639`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2401`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1117-1119`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxPlayEffectIdArgs {
    id: c_int,
    org: *const vec3_t,
    fwd: *const vec3_t,
    vol: c_int,
    rad: c_int,
}

impl CgFxPlayEffectIdArgs {
    pub const fn new(
        id: c_int,
        org: *const vec3_t,
        fwd: *const vec3_t,
        vol: c_int,
        rad: c_int,
    ) -> Self {
        Self {
            id,
            org,
            fwd,
            vol,
            rad,
        }
    }
}

/// `CG_FX_PLAY_EFFECT_ID` MP cgame imports syscall ABI token.
///
/// Raven: builds arbitrary perp. right vector, does a cross product to define up.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:222`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:637-639`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1117-1119`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1117-1119`
pub struct CgFxPlayEffectId;

impl OutboundSysCall for CgFxPlayEffectId {
    type Import = MpCgameImport;
    type Args = CgFxPlayEffectIdArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_PLAY_EFFECT_ID;
}

impl EncodeSysCall for CgFxPlayEffectId {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.id as isize,
            ptr_to_word(args.org),
            ptr_to_word(args.fwd),
            args.vol as isize,
            args.rad as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFxPlayEffectId {
    fn decode_return(_word: isize) -> Self::Output {}
}
