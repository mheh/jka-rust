use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_ROFF_NOTETRACK_CALLBACK` MP cgame exports vmMain boundary token.
///
/// Raven: int entnum, char *notetrack
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:424`
pub struct CgRoffNotetrackCallback;

impl InboundVmCall for CgRoffNotetrackCallback {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_ROFF_NOTETRACK_CALLBACK;
}
