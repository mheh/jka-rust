use core::ffi::c_int;

use super::super::MpCgameImport;
use abi_transport::generic::{
    ptr_to_word, DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport,
};
use mp_qshared::shared::vec3_t;

/// Arguments for `CG_FX_PLAY_PORTAL_EFFECT_ID`.
///
/// Raven wrapper: `syscall( CG_FX_PLAY_PORTAL_EFFECT_ID, id, org, fwd);`
/// Raven transport: `FX_PlayEffectID(args[1], (float *)VMA(2), (float *)VMA(3), args[4], args[5], qtrue ); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:642-644`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2402`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1121-1123`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxPlayPortalEffectIdArgs {
    id: c_int,
    org: *const vec3_t,
    fwd: *const vec3_t,
}

impl CgFxPlayPortalEffectIdArgs {
    pub const fn new(id: c_int, org: *const vec3_t, fwd: *const vec3_t) -> Self {
        Self { id, org, fwd }
    }
}

/// `CG_FX_PLAY_PORTAL_EFFECT_ID` MP cgame imports syscall ABI token.
///
/// Raven: wrapper omits the header's `vol` and `rad`; this type matches the actual syscall transport.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:223`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:642-644`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1121-1123`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1121-1123`
pub struct CgFxPlayPortalEffectId;

impl OutboundSysCall for CgFxPlayPortalEffectId {
    type Import = MpCgameImport;
    type Args = CgFxPlayPortalEffectIdArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_PLAY_PORTAL_EFFECT_ID;
}

impl EncodeSysCall for CgFxPlayPortalEffectId {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([
            args.id as isize,
            ptr_to_word(args.org),
            ptr_to_word(args.fwd),
        ])
    }
}

impl DecodeSysCallReturn for CgFxPlayPortalEffectId {
    fn decode_return(_word: isize) -> Self::Output {}
}
