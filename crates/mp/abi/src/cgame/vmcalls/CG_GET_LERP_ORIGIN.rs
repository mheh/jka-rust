// Shared-buffer payload lives in cg.sharedBuffer / cl.mSharedMemory, not vmMain arg slots.
use super::super::shared_buffer::{SharedBufferPayload, TCGVectorData};
use super::super::MpCgameExport;
use abi_transport::generic::InboundVmCall;

/// `CG_GET_LERP_ORIGIN` MP cgame exports vmMain ABI token.
///
/// Raven: void CG_LerpOrigin(int num, vec3_t result);
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:395-396`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:484-489`
/// Args/source source: `oracle/oracle/codemp/cgame/cg_main.c:223-225`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:373-374`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:223-225`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxScheduler.cpp:130-133`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxScheduler.cpp:929-933`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxScheduler.cpp:1104-1105`
/// Shared-buffer payload type source: `oracle/oracle/codemp/cgame/cg_public.h:484-489`
pub struct CgGetLerpOrigin;

impl InboundVmCall for CgGetLerpOrigin {
    type Command = MpCgameExport;
    type Args = SharedBufferPayload<TCGVectorData>;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_GET_LERP_ORIGIN;
}
