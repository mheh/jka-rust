use super::super::MpCgameExport;
use crate::boundary::generic::{EncodeVmMainReturn, InboundVmCall};

/// `CG_FX_CAMERASHAKE` MP cgame exports vmMain boundary token.
///
/// Raven: mcg post-gold added
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:439`
/// Args source: `oracle/oracle/codemp/cgame/cg_public.h:512-519`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:346-352`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:346-352`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxSystem.cpp:84-92`
pub struct CgFxCamerashake;

impl InboundVmCall for CgFxCamerashake {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args; payload is TCGCameraShake in cg.sharedBuffer/cl.mSharedMemory.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_FX_CAMERASHAKE;
}

impl EncodeVmMainReturn for CgFxCamerashake {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
