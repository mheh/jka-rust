// Shared-buffer payload lives in cg.sharedBuffer / cl.mSharedMemory, not vmMain arg slots.
use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_GET_LERP_ORIGIN` MP cgame exports vmMain boundary token.
///
/// Raven: void CG_LerpOrigin(int num, vec3_t result);
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:395-396`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:369-373`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:223-225`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxScheduler.cpp:130-133`
pub struct CgGetLerpOrigin;

impl InboundVmCall for CgGetLerpOrigin {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args; payload is TCGVectorData in cg.sharedBuffer/cl.mSharedMemory.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_LERP_ORIGIN;
}
