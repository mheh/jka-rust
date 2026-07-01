use super::super::shared_buffer::{SharedBufferPayload, TCGTrace};
use super::super::MpCgameExport;
use abi_transport::generic::InboundVmCall;

/// `CG_G2TRACE` MP cgame exports vmMain ABI token.
///
/// Raven: void CG_Trace( trace_t *result, const vec3_t start, const vec3_t mins, const vec3_t maxs,
/// Raven: const vec3_t end, int skipNumber, int mask );
/// Raven: shared-buffer payload `TCGTrace` carries `mResult` output and start/mins/maxs/end/skip/mask inputs.
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:405-408`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:490-496`
/// Args/source source: `oracle/oracle/codemp/cgame/cg_main.c:248-250`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:248-250`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:248-250`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxSystem.h:109-131`, `oracle/oracle/codemp/client/FxSystem.h:134-158`
/// Shared-buffer payload type source: `oracle/oracle/codemp/cgame/cg_public.h:490-496`
pub struct CgG2trace;

impl InboundVmCall for CgG2trace {
    type Command = MpCgameExport;
    type Args = SharedBufferPayload<TCGTrace>;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_G2TRACE;
}
