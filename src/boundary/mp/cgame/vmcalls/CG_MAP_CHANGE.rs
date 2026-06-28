use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_MAP_CHANGE` MP cgame exports vmMain boundary token.
///
/// Raven: void CG_ImpactMark( qhandle_t markShader, const vec3_t origin, const vec3_t dir,
/// Raven: float orientation, float red, float green, float blue, float alpha,
/// Raven: qboolean alphaFade, float radius, qboolean temporary )
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:431`
pub struct CgMapChange;

impl InboundVmCall for CgMapChange {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_MAP_CHANGE;
}
