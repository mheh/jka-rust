use super::super::MpCgameExport;
use crate::boundary::generic::InboundVmCall;

/// `CG_SHUTDOWN` MP cgame exports vmMain boundary token.
///
/// Raven: void CG_Init( int serverMessageNum, int serverCommandSequence, int clientNum )
/// Raven: called when the level loads or when the renderer is restarted
/// Raven: all media should be registered at this time
/// Raven: cgame will display loading status by calling SCR_Update, which
/// Raven: will call CG_DrawInformation during the loading process
/// Raven: reliableCommandSequence will be 0 on fresh loads, but higher for
/// Raven: demos, tourney restarts, or vid_restarts
/// Source: `oracle/oracle/codemp/cgame/cg_public.h:362`
pub struct CgShutdown;

impl InboundVmCall for CgShutdown {
    type Command = MpCgameExport;
    type Args = (); //TODO: Port args
    type Output = (); //TODO: Port output

    const COMMAND: MpCgameExport = MpCgameExport::CG_SHUTDOWN;
}
