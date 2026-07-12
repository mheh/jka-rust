use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_FX_PLAY_ENTITY_EFFECT_ID`.
///
/// Raven wrapper: `syscall( CG_FX_PLAY_ENTITY_EFFECT_ID, id, org, axis, boltInfo, entNum, vol, rad );`
/// Raven transport: `FX_PlayEntityEffectID(args[1], (float *)VMA(2), (vec3_t *)VMA(3), args[4], args[5], args[6], args[7] ); return 0;`
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:647-650`
/// Args source: `oracle/codemp/cgame/cg_local.h:2403`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1125-1127`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxPlayEntityEffectIdArgs {
    id: c_int,
    org: *const vec3_t,
    axis: *const vec3_t,
    bolt_info: c_int,
    ent_num: c_int,
    vol: c_int,
    rad: c_int,
}

impl CgFxPlayEntityEffectIdArgs {
    pub const fn new(
        id: c_int,
        org: *const vec3_t,
        axis: *const vec3_t,
        bolt_info: c_int,
        ent_num: c_int,
        vol: c_int,
        rad: c_int,
    ) -> Self {
        Self {
            id,
            org,
            axis,
            bolt_info,
            ent_num,
            vol,
            rad,
        }
    }
}

/// `CG_FX_PLAY_ENTITY_EFFECT_ID` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:224`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:647-650`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1125-1127`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1125-1127`
pub struct CgFxPlayEntityEffectId;

impl OutboundSysCall for CgFxPlayEntityEffectId {
    type Import = MpCgameImport;
    type Args = CgFxPlayEntityEffectIdArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_PLAY_ENTITY_EFFECT_ID;
}

impl EncodeSysCall for CgFxPlayEntityEffectId {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.id as isize,
            ptr_to_word(args.org),
            ptr_to_word(args.axis),
            args.bolt_info as isize,
            args.ent_num as isize,
            args.vol as isize,
            args.rad as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFxPlayEntityEffectId {
    fn decode_return(_word: isize) -> Self::Output {}
}
