use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_CROSSHAIR_PLAYER` MP cgame exports vmMain boundary token.
///
/// Raven: void (*CG_DrawActiveFrame)( int serverTime, stereoFrame_t stereoView, qboolean demoPlayback );
/// Raven: Generates and draws a game scene and status information at the given time.
/// Raven: If demoPlayback is set, local movement prediction will not be enabled
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:378`
pub struct CgCrosshairPlayer;

impl InboundVmCall for CgCrosshairPlayer {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_CROSSHAIR_PLAYER;
}
