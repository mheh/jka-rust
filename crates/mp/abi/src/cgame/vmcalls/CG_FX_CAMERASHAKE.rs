use super::super::shared_buffer::{SharedBufferPayload, TCGCameraShake};
use super::super::MpCgameExport;
use abi_transport::generic::{EncodeVmMainReturn, InboundVmCall};

/// `CG_FX_CAMERASHAKE` MP cgame exports vmMain ABI token.
///
/// Raven: mcg post-gold added
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:439`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:346-352`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:512-519`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:346-351`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:346-352`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxSystem.cpp:85-93`
/// Shared-buffer payload type source: `oracle/oracle/codemp/cgame/cg_public.h:512-519`
pub struct CgFxCamerashake;

impl InboundVmCall for CgFxCamerashake {
    type Command = MpCgameExport;
    type Args = SharedBufferPayload<TCGCameraShake>;
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_FX_CAMERASHAKE;
}

impl EncodeVmMainReturn for CgFxCamerashake {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
