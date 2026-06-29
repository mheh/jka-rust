use super::super::MpCgameExport;
use crate::abi::generic::InboundVmCall;

/// `CG_TRACE` MP cgame exports vmMain ABI token.
///
/// Raven: void CG_Trace( trace_t *result, const vec3_t start, const vec3_t mins, const vec3_t maxs,
/// Raven: const vec3_t end, int skipNumber, int mask );
/// Raven: shared-buffer payload `TCGTrace` carries `mResult` output and start/mins/maxs/end/skip/mask inputs.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:405-408`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:490-496`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:243-245`, `oracle/oracle/codemp/cgame/cg_main.c:408-413`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:243-245`, `oracle/oracle/codemp/cgame/cg_main.c:413`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:243-245`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxSystem.h:109-131`, `oracle/oracle/codemp/RMG/RM_Terrain.cpp:423-432`
/// Shared-buffer payload type source: `oracle/oracle/codemp/cgame/cg_public.h:490-496`
/// FIXME: create type `TCGTrace` in Rust from
/// `oracle/oracle/codemp/cgame/cg_public.h:490-496`.
pub struct CgTrace;

impl InboundVmCall for CgTrace {
    type Command = MpCgameExport;
    type Args = (); //FIXME: create type `TCGTrace` in Rust when payload modeling is added.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_TRACE;
}
