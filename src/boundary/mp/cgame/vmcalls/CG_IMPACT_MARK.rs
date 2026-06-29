use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_IMPACT_MARK` MP cgame exports vmMain boundary token.
///
/// Raven: void CG_ImpactMark(qhandle_t markShader, const vec3_t origin, const vec3_t dir, float orientation,
/// Raven: float red, float green, float blue, float alpha, qboolean alphaFade, float radius, qboolean temporary);
/// Raven: shared-buffer payload `TCGImpactMark` carries the mark shader, origin, dir, orientation/color/radius,
/// Raven: and temporary/alpha-fade flags.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:426-429`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:467-479`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:303-305`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:570+`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:303-305`
/// Transport/call-site source: TODO exact engine caller not found; see `oracle/oracle/codemp/cgame/cg_main.c:303-305`
/// Transport/call-site source: TODO exact engine caller not found; see `oracle/oracle/codemp/cgame/cg_main.c:570+`
pub struct CgImpactMark;

impl InboundVmCall for CgImpactMark {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args; payload is TCGImpactMark in cg.sharedBuffer/cl.mSharedMemory.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_IMPACT_MARK;
}
