use super::super::MpCgameImport;
use crate::abi::generic::{DecodeSysCallReturn, EncodeSysCall, OutboundSysCall, SysCallTransport};
use crate::ffi::types::qboolean;

/// Arguments for `CG_FX_ADD_SCHEDULED_EFFECTS`.
///
/// Raven wrapper: `syscall( CG_FX_ADD_SCHEDULED_EFFECTS, skyPortal );`
/// Raven transport: `FX_AddScheduledEffects((qboolean)args[1]); return 0;`
///
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:659-661`
/// Args source: `oracle/oracle/codemp/cgame/cg_local.h:2405`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1141-1143`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgFxAddScheduledEffectsArgs {
    sky_portal: qboolean,
}

impl CgFxAddScheduledEffectsArgs {
    pub const fn new(sky_portal: qboolean) -> Self {
        Self { sky_portal }
    }
}

/// `CG_FX_ADD_SCHEDULED_EFFECTS` MP cgame imports syscall ABI token.
///
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:226`
/// Args source: `oracle/oracle/codemp/cgame/cg_syscalls.c:659-661`
/// Output source: `oracle/oracle/codemp/client/cl_cgame.cpp:1141-1143`
/// Transport/switch source: `oracle/oracle/codemp/client/cl_cgame.cpp:1141-1143`
pub struct CgFxAddScheduledEffects;

impl OutboundSysCall for CgFxAddScheduledEffects {
    type Import = MpCgameImport;
    type Args = CgFxAddScheduledEffectsArgs;
    type Output = ();

    const IMPORT: MpCgameImport = MpCgameImport::CG_FX_ADD_SCHEDULED_EFFECTS;
}

impl EncodeSysCall for CgFxAddScheduledEffects {
    fn encode_syscall(args: &Self::Args) -> SysCallTransport {
        SysCallTransport::new([args.sky_portal as isize])
    }
}

impl DecodeSysCallReturn for CgFxAddScheduledEffects {
    fn decode_return(_word: isize) -> Self::Output {}
}
