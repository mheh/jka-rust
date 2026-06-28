use super::super::SpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_DRAW_DATAPAD_HUD` SP cgame exports vmMain boundary token.
///
/// Raven: Ghoul2 Insert End
/// Source: `oracle/oracle/code/client/vmachine.h:33`
pub struct CgDrawDatapadHud;

impl InboundVmCall for CgDrawDatapadHud {
    type Command = SpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: SpCgameExport = SpCgameExport::CG_DRAW_DATAPAD_HUD;
}
