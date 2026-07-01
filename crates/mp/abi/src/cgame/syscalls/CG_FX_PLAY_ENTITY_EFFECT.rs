use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_FX_PLAY_ENTITY_EFFECT`.
///
/// Raven wrapper: `syscall( CG_FX_PLAY_ENTITY_EFFECT, file, org, axis, boltInfo, entNum, vol, rad );`
/// Raven transport: `assert(0);//gone!` then returns `0`; the original transport line is left commented out.
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:631-634`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2400`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1112-1115`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxPlayEntityEffectArgs {
    file: *const c_char,
    org: *const vec3_t,
    axis: *const vec3_t,
    bolt_info: c_int,
    ent_num: c_int,
    vol: c_int,
    rad: c_int,
}

impl CgFxPlayEntityEffectArgs {
    pub const fn new(
        file: *const c_char,
        org: *const vec3_t,
        axis: *const vec3_t,
        bolt_info: c_int,
        ent_num: c_int,
        vol: c_int,
        rad: c_int,
    ) -> Self {
        Self {
            file,
            org,
            axis,
            bolt_info,
            ent_num,
            vol,
            rad,
        }
    }
}

/// `CG_FX_PLAY_ENTITY_EFFECT` MP cgame imports syscall ABI token.
///
/// Raven: this engine switch arm is marked `assert(0);//gone!` and returns `0`.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:221`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:631-634`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1112-1115`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1112-1115`
pub struct CgFxPlayEntityEffect;

impl OutboundSysCall for CgFxPlayEntityEffect {
    type Import = MpCgameImport;
    type Args = CgFxPlayEntityEffectArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_PLAY_ENTITY_EFFECT;
}

impl EncodeSysCall for CgFxPlayEntityEffect {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.file),
            ptr_to_word(args.org),
            ptr_to_word(args.axis),
            args.bolt_info as isize,
            args.ent_num as isize,
            args.vol as isize,
            args.rad as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFxPlayEntityEffect {
    fn decode_return(_word: isize) -> Self::Output {}
}
