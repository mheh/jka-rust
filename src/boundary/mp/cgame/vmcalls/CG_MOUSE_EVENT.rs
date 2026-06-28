use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_MOUSE_EVENT` MP cgame exports vmMain boundary token.
///
/// Raven: void	(*CG_KeyEvent)( int key, qboolean down );
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:387`
pub struct CgMouseEvent;

impl InboundVmCall for CgMouseEvent {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_MOUSE_EVENT;
}
