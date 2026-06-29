use super::super::MpCgameExport;
use crate::abi::generic::InboundVmCall;

/// `CG_IMPACT_MARK` MP cgame exports vmMain ABI token.
///
/// Raven: void CG_ImpactMark(qhandle_t markShader, const vec3_t origin, const vec3_t dir, float orientation,
/// Raven: float red, float green, float blue, float alpha, qboolean alphaFade, float radius, qboolean temporary);
/// Raven: shared-buffer payload `TCGImpactMark` carries the mark shader, origin, dir, orientation/color/radius,
/// Raven: and temporary/alpha-fade flags.
/// Enum source: `oracle/oracle/codemp/cgame/cg_public.h:426-429`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:467-479`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:303-305`, `oracle/oracle/codemp/cgame/cg_main.c:570-580`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:303-305`, `oracle/oracle/codemp/cgame/cg_main.c:578-579`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:303-305`
/// Transport/call-site source: no `VM_Call( cgvm, CG_IMPACT_MARK )` site found in tracked Oracle sources; dispatch and payload usage is established in `oracle/oracle/codemp/cgame/cg_main.c`.
/// Shared-buffer payload type source: `oracle/oracle/codemp/cgame/cg_public.h:467-479`
/// FIXME: create type `TCGImpactMark` in Rust from
/// `oracle/oracle/codemp/cgame/cg_public.h:467-479`.
pub struct CgImpactMark;

impl InboundVmCall for CgImpactMark {
    type Command = MpCgameExport;
    type Args = (); //FIXME: create type `TCGImpactMark` in Rust when this payload is modeled.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_IMPACT_MARK;
}
