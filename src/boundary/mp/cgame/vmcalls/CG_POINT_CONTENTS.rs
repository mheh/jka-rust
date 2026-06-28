use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_POINT_CONTENTS` MP cgame exports vmMain boundary token.
///
/// Raven: void (*CG_EventHandling)(int type);
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:392`
pub struct CgPointContents;

impl InboundVmCall for CgPointContents {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_POINT_CONTENTS;
}
