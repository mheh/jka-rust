use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_FX_CAMERASHAKE` MP cgame exports vmMain boundary token.
///
/// Raven: mcg post-gold added
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:439`
pub struct CgFxCamerashake;

impl InboundVmCall for CgFxCamerashake {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_FX_CAMERASHAKE;
}
