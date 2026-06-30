use core::ffi::{c_char, c_int};

use super::super::MpCgameImport;
use crate::abi::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use crate::shared::vec3_t;

/// Arguments for `CG_FX_PLAY_EFFECT`.
///
/// Raven wrapper: `syscall( CG_FX_PLAY_EFFECT, file, org, fwd, vol, rad);`
/// Raven transport: `FX_PlayEffect((const char *)VMA(1), (float *)VMA(2), (float *)VMA(3), args[4], args[5] ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:626-628`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2399`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1108-1110`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxPlayEffectArgs {
    file: *const c_char,
    org: *const vec3_t,
    fwd: *const vec3_t,
    vol: c_int,
    rad: c_int,
}

impl CgFxPlayEffectArgs {
    pub const fn new(
        file: *const c_char,
        org: *const vec3_t,
        fwd: *const vec3_t,
        vol: c_int,
        rad: c_int,
    ) -> Self {
        Self {
            file,
            org,
            fwd,
            vol,
            rad,
        }
    }
}

/// `CG_FX_PLAY_EFFECT` MP cgame imports syscall ABI token.
///
/// Raven: builds arbitrary perp. right vector, does a cross product to define up.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:220`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:626-628`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1108-1110`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1108-1110`
pub struct CgFxPlayEffect;

impl OutboundSysCall for CgFxPlayEffect {
    type Import = MpCgameImport;
    type Args = CgFxPlayEffectArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_PLAY_EFFECT;
}

impl EncodeSysCall for CgFxPlayEffect {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            ptr_to_word(args.file),
            ptr_to_word(args.org),
            ptr_to_word(args.fwd),
            args.vol as isize,
            args.rad as isize,
        ])
    }
}

impl DecodeSysCallReturn for CgFxPlayEffect {
    fn decode_return(_word: isize) -> Self::Output {}
}
