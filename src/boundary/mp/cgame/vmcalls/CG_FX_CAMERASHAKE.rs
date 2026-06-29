use super::super::MpCgameExport;
use crate::boundary::generic::{EncodeVmMainReturn, InboundVmCall};

/// `CG_FX_CAMERASHAKE` MP cgame exports vmMain boundary token.
///
/// Raven: mcg post-gold added
/// Enum value source: `oracle/oracle/codemp/cgame/cg_public.h:439`
/// Args source: `oracle/oracle/codemp/cgame/cg_main.c:346-352`
/// Shared-buffer source: `oracle/oracle/codemp/cgame/cg_public.h:512-519`
/// Output source: `oracle/oracle/codemp/cgame/cg_main.c:346-351`
/// Transport/switch source: `oracle/oracle/codemp/cgame/cg_main.c:346-352`
/// Transport/call-site source: `oracle/oracle/codemp/client/FxSystem.cpp:85-93`
/// Shared-buffer payload type is not represented in Rust yet.
/// FIXME: create type `TCGCameraShake` from
/// `oracle/oracle/codemp/cgame/cg_public.h:512-519`.
pub struct CgFxCamerashake;

impl InboundVmCall for CgFxCamerashake {
    type Command = MpCgameExport;
    type Args = (); //FIXME: create type `TCGCameraShake` in Rust when this payload moves to first-class ABI args.
    type Output = ();

    const COMMAND: MpCgameExport = MpCgameExport::CG_FX_CAMERASHAKE;
}

impl EncodeVmMainReturn for CgFxCamerashake {
    fn encode_return(_output: Self::Output) -> isize {
        0
    }
}
