use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_EVENT_HANDLING` MP cgame exports vmMain boundary token.
///
/// Raven: void	(*CG_MouseEvent)( int dx, int dy );
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:389`
pub struct CgEventHandling;

impl InboundVmCall for CgEventHandling {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_EVENT_HANDLING;
}
