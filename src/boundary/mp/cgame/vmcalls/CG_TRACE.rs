use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_TRACE` MP cgame exports vmMain boundary token.
///
/// Raven: void CG_Trace( trace_t *result, const vec3_t start, const vec3_t mins, const vec3_t maxs,
/// Raven: const vec3_t end, int skipNumber, int mask );
/// Raven: shared-buffer payload `TCGTrace` carries `mResult` output and start/mins/maxs/end/skip/mask inputs.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:405-408`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:490-496`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:243-245`
/// Args source: `oracle/oracle/codemp/client/FxSystem.h:107-131`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:243-245`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxSystem.h:107-131`
pub struct CgTrace;

impl InboundVmCall for CgTrace {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args; payload is TCGTrace in cg.sharedBuffer/cl.mSharedMemory.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_TRACE;
}
