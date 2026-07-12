use core::ffi::{c_int, c_void};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::{qboolean, vec3_t};

/// Arguments for `CG_FX_PLAY_BOLTED_EFFECT_ID`.
///
/// Raven wrapper: `syscall( CG_FX_PLAY_BOLTED_EFFECT_ID, id, org, ghoul2, boltNum, entNum, modelNum, iLooptime, isRelative );`
/// Raven transport attaches the Ghoul2 entity and returns `1` or `0`; the cgame wrapper ignores that return word.
///
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:653-656`
/// Args source: `oracle/codemp/cgame/cg_local.h:2404`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1129-1139`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxPlayBoltedEffectIdArgs {
    id: c_int,
    org: *const vec3_t,
    ghoul2: *mut c_void,
    bolt_num: c_int,
    ent_num: c_int,
    model_num: c_int,
    i_loop_time: c_int,
    is_relative: qboolean,
}

impl CgFxPlayBoltedEffectIdArgs {
    pub const fn new(
        id: c_int,
        org: *const vec3_t,
        ghoul2: *mut c_void,
        bolt_num: c_int,
        ent_num: c_int,
        model_num: c_int,
        i_loop_time: c_int,
        is_relative: qboolean,
    ) -> Self {
        Self {
            id,
            org,
            ghoul2,
            bolt_num,
            ent_num,
            model_num,
            i_loop_time,
            is_relative,
        }
    }
}

/// `CG_FX_PLAY_BOLTED_EFFECT_ID` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/codemp/cgame/cg_public.h:225`
/// Args source: `oracle/codemp/cgame/cg_syscalls.c:653-656`
/// Output source: `oracle/codemp/cgame/cg_syscalls.c:653-657`
/// Output source: `oracle/codemp/client/cl_cgame.cpp:1129-1139`
/// Transport/switch source: `oracle/codemp/client/cl_cgame.cpp:1129-1139`
pub struct CgFxPlayBoltedEffectId;

impl OutboundSysCall for CgFxPlayBoltedEffectId {
    type Import = MpCgameImport;
    type Args = CgFxPlayBoltedEffectIdArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_PLAY_BOLTED_EFFECT_ID;
}

impl EncodeSysCall for CgFxPlayBoltedEffectId {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.id as isize,
            ptr_to_word(args.org),
            ptr_to_word(args.ghoul2),
            args.bolt_num as isize,
            args.ent_num as isize,
            args.model_num as isize,
            args.i_loop_time as isize,
            args.is_relative as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFxPlayBoltedEffectId {
    fn decode_return(_word: isize) -> Self::Output {}
}
